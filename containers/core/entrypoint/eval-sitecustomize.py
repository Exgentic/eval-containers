"""Eval Containers sitecustomize — authenticates HuggingFace downloads and
retries urllib on transient failures.

Canonical home: `core/entrypoint/eval-sitecustomize.py`. Any image that needs
it does:

    COPY --from=ghcr.io/exgentic/core/entrypoint:latest \\
         /eval-sitecustomize.py \\
         /usr/local/lib/python3.12/site-packages/sitecustomize.py

One file, every benchmark/agent image (rule 11). Benchmarks keep using stock
`urllib.request.urlopen` / `urllib.request.urlretrieve`; Python's `site` module
auto-loads this at interpreter startup, transparently adding two behaviours:

1. **HuggingFace auth.** Most benchmarks download datasets from huggingface.co
   with no auth header, so they hit the low *anonymous* rate limit — the main
   source of build-time 429 storms under concurrent builds. When an HF_TOKEN is
   present, requests to huggingface.co get an `Authorization: Bearer` header
   (lifting the rate limit ~10×). Implemented as a global opener so it covers
   both `urlopen` and `urlretrieve` without touching the URL the caller passed.
   Only huggingface.co is authenticated — the signed CDN redirect is a different
   host and is left alone — and an existing Authorization header is never
   clobbered. The token is read from the env or the build-secret mount at
   `/run/secrets/HF_TOKEN`; it is never written to a layer (the RUN that needs
   it mounts `type=secret`, so the token only exists during that step). At eval
   runtime no secret is mounted, so downloads stay anonymous exactly as before.

2. **Retry.** Transient network errors and retryable HTTP statuses (429 + 5xx)
   are retried with exponential backoff, honouring a `Retry-After` header when
   the server sends one. Other 4xx (404/401/403) are real — fail fast rather
   than burn the backoff budget on a genuinely missing/forbidden resource.

Why retry at all: 68 of 84 build failures observed 2026-04-17 came from
benchmarks doing `urllib.request.urlretrieve(HF_URL, path)` inside a
`RUN python3 <<'PYEOF'` heredoc. Under concurrent builds, HuggingFace returns
partial bodies, resets the TCP connection, or 429s; the default stdlib gives up
on the first hiccup.

Tunables (env vars read at interpreter startup):
  EVAL_NET_RETRIES         integer, default 6
  EVAL_NET_BACKOFF         seconds base, default 5 (delay = BACKOFF * attempt)
  EVAL_NET_TIMEOUT         seconds, default 120 (socket.setdefaulttimeout backstop)
  EVAL_NET_RETRY_AFTER_CAP seconds, default 60 (cap on a server-sent Retry-After)
"""

import http.client
import os
import socket
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

_MAX_ATTEMPTS = int(os.environ.get("EVAL_NET_RETRIES", "6"))
_BACKOFF = float(os.environ.get("EVAL_NET_BACKOFF", "5"))
_TIMEOUT = float(os.environ.get("EVAL_NET_TIMEOUT", "120"))
_RETRY_AFTER_CAP = float(os.environ.get("EVAL_NET_RETRY_AFTER_CAP", "60"))

# Non-HTTP transient errors worth retrying. HTTP status errors are classified
# separately (see _retry) so that real 4xx fail fast instead of retrying.
_TRANSIENT = (
    TimeoutError,
    socket.timeout,
    socket.gaierror,
    ConnectionError,
    http.client.IncompleteRead,
    http.client.BadStatusLine,
    http.client.RemoteDisconnected,
    urllib.error.URLError,
)

# Rate-limit + server-side transients. Other 4xx are real bugs -> fail fast.
_RETRYABLE_STATUS = {429, 500, 502, 503, 504}


def _hf_token():
    """The HuggingFace token from the env or the build-secret mount, or None."""
    for var in ("HF_TOKEN", "HUGGING_FACE_HUB_TOKEN", "HUGGINGFACE_TOKEN"):
        val = os.environ.get(var)
        if val and val.strip():
            return val.strip()
    try:
        with open("/run/secrets/HF_TOKEN") as fh:  # BuildKit `type=secret` mount
            return fh.read().strip() or None
    except OSError:
        return None


# Read once at interpreter startup: during a build RUN the secret is mounted;
# at eval runtime it is absent (-> None -> anonymous, unchanged).
_HF_TOKEN = _hf_token()


class _HFAuthHandler(urllib.request.BaseHandler):
    """Attach `Authorization: Bearer $HF_TOKEN` to huggingface.co requests.

    Runs as a request processor inside the installed opener, so it covers
    `urlopen` and (via its internal `urlopen`) `urlretrieve` without the caller
    passing a Request object.
    """

    def http_request(self, req):
        if _HF_TOKEN and not req.has_header("Authorization"):
            host = (urllib.parse.urlsplit(req.full_url).hostname or "").lower()
            # Exact host only: the api/resolve endpoints need the token; the
            # signed CDN redirect (cdn-lfs.huggingface.co) must not get it.
            if host == "huggingface.co":
                req.add_header("Authorization", "Bearer " + _HF_TOKEN)
        return req

    https_request = http_request


_original_urlretrieve = urllib.request.urlretrieve
_original_urlopen = urllib.request.urlopen


def _delay(exc, attempt):
    """Seconds before the next attempt — server Retry-After (capped) or backoff."""
    if isinstance(exc, urllib.error.HTTPError) and exc.headers is not None:
        hdr = exc.headers.get("Retry-After")
        if hdr:
            try:
                return min(float(hdr), _RETRY_AFTER_CAP)
            except ValueError:
                pass  # HTTP-date form — fall back to exponential backoff
    return _BACKOFF * attempt


def _retry(fn):
    def wrapped(*args, **kwargs):
        for attempt in range(1, _MAX_ATTEMPTS + 1):
            try:
                return fn(*args, **kwargs)
            except urllib.error.HTTPError as e:
                # 429 + 5xx are transient; other 4xx are real -> fail fast.
                if e.code not in _RETRYABLE_STATUS or attempt == _MAX_ATTEMPTS:
                    raise
                delay = _delay(e, attempt)
                print(
                    f"[eval-retry] {fn.__name__} attempt {attempt}/{_MAX_ATTEMPTS} "
                    f"HTTP {e.code}; retry in {delay:.0f}s",
                    file=sys.stderr,
                    flush=True,
                )
                time.sleep(delay)
            except _TRANSIENT as e:
                if attempt == _MAX_ATTEMPTS:
                    raise
                delay = _BACKOFF * attempt
                print(
                    f"[eval-retry] {fn.__name__} attempt {attempt}/{_MAX_ATTEMPTS} "
                    f"failed: {type(e).__name__}: {e}; retry in {delay:.0f}s",
                    file=sys.stderr,
                    flush=True,
                )
                time.sleep(delay)

    wrapped.__name__ = fn.__name__
    wrapped.__doc__ = fn.__doc__
    return wrapped


# Auth via the global opener (covers urlopen + urlretrieve's internal urlopen).
urllib.request.install_opener(urllib.request.build_opener(_HFAuthHandler()))

# Backstop: no single connection hangs forever.
socket.setdefaulttimeout(_TIMEOUT)

# Retry wraps both stdlib entry points (urlretrieve also retries its body read).
urllib.request.urlretrieve = _retry(_original_urlretrieve)
urllib.request.urlopen = _retry(_original_urlopen)
