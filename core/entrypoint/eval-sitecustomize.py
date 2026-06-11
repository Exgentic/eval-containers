"""Eval Containers sitecustomize — retries urllib on transient network failures.

Canonical home: `core/entrypoint/eval-sitecustomize.py`. Any image that
needs urllib retries does:

    COPY --from=ghcr.io/exgentic/core/entrypoint:latest \\
         /eval-sitecustomize.py \\
         /usr/local/lib/python3.12/site-packages/sitecustomize.py

One file, every benchmark/agent image (rule 11).

Why: 68 of 84 build failures observed 2026-04-17 came from benchmarks
doing `urllib.request.urlretrieve(HF_URL, path)` inside a
`RUN python3 <<'PYEOF'` heredoc. Under concurrent builds, HuggingFace
returns partial bodies or resets the TCP connection; the default stdlib
gives up on the first hiccup.

Benchmarks keep using stock `urllib.request.urlretrieve` /
`urllib.request.urlopen`. Python's `site` module auto-loads this file at
interpreter startup if it lands on sys.path as `sitecustomize.py`,
silently upgrading both to retry up to 6 times with exponential backoff
on the transient exceptions the stdlib raises.

Tunables (env vars read at interpreter startup):
  EVAL_NET_RETRIES  integer, default 6
  EVAL_NET_BACKOFF  seconds base, default 5 (delay = BACKOFF * attempt)
  EVAL_NET_TIMEOUT  seconds, default 120 (socket.setdefaulttimeout backstop)
"""

import http.client
import os
import socket
import sys
import time
import urllib.error
import urllib.request

_MAX_ATTEMPTS = int(os.environ.get("EVAL_NET_RETRIES", "6"))
_BACKOFF = float(os.environ.get("EVAL_NET_BACKOFF", "5"))
_TIMEOUT = float(os.environ.get("EVAL_NET_TIMEOUT", "120"))

# Transient network errors that are worth retrying. Anything outside this
# tuple (HTTPError 4xx, ValueError, etc) is a real bug — fail fast.
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

_original_urlretrieve = urllib.request.urlretrieve
_original_urlopen = urllib.request.urlopen


def _retry(fn):
    def wrapped(*args, **kwargs):
        for attempt in range(1, _MAX_ATTEMPTS + 1):
            try:
                return fn(*args, **kwargs)
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


# Backstop: no single connection hangs forever.
socket.setdefaulttimeout(_TIMEOUT)

urllib.request.urlretrieve = _retry(_original_urlretrieve)
urllib.request.urlopen = _retry(_original_urlopen)
