#!/usr/bin/env python3
"""fleet-retention — a census of every version the registry holds, and which of
them nothing points at any more (delivery/RULES.md 22).

Rule 22 says a published digest is retained while `latest`, a release, or a
published artifact resolves to it, and for 90 days after that ceases. Nothing
implemented it and nothing could even measure it: ~9,900 packages, ~1.5M
versions, and no way to tell a superseded digest from a live one. This writes
the census down so a human can review it. It deletes nothing.

WHAT MAKES THIS SAFE: an untagged version is NOT a dangling one. A multi-arch
image is an index whose per-arch and attestation children are themselves
versions, and those children carry no tags of their own — so the arm64 half of
the `latest` everyone pulls looks exactly like garbage to a "delete untagged"
sweep. Measured on benchmarks/aime: of 4 children of the live `:latest`, 3 are
untagged versions. Reachability is therefore computed, never assumed — every
tagged version is expanded into its children and those are subtracted:

    reachable(P)  = tagged(P) ∪ ⋃ children(every tagged version)
    candidates(P) = versions(P) \\ reachable(P)

Per-version verdicts. Only `aged` is a candidate for review; everything else is
retained, and `aged` is NOT a clearance to delete (see THE CLOCK):

  tagged      carries any tag — latest*, a hash tag, a SemVer     (rules 18-20)
  child       untagged, but a child of a tagged index — per-arch
              OR attestation. The verdict a date-only sweep misses (rule 22)
  recent      unreachable, but created inside the 90-day window   (rule 22)
  aged        unreachable and created before the cutoff — a review candidate
  cache       a buildcache/* version, outside rule 22 entirely
  unreadable  a tagged version's manifest never read, so its children are
              UNKNOWN — the whole package is withheld       (rule 14, fail dirty)

THE CLOCK IS A PROXY, AND IT IS NOT SOUND. Rule 22's 90 days run from when a
digest stopped being referenced. The packages API records no such field —
`updated_at` equals `created_at` on every version measured, so it carries no
supersession signal. So `aged` is keyed on creation date, which over-selects: a
digest created 100 days ago may have held `latest` until yesterday, and would
still read `aged`. A manifest from this tool is a census for review, never an
authorisation to delete; `clock.sound: false` says so in the output. Making the
clock sound needs the publisher to record supersession when it moves a tag —
a rule change, not something this tool can infer.

Two APIs, because each is blind where the other sees:
  - the packages REST API has version ids and dates, but is rate-limited
    (5,000/hr, shared) and its package *list* truncates at per_page*page >
    10000 — so a full sweep cannot prove it saw every package, and says so.
  - the registry /v2/ API resolves a manifest's children and costs zero GitHub
    budget (measured in fleet-index.py), but lists only TAGS — 8 for a package
    holding 167 versions — so it can never enumerate what to review.

Usage:
  fleet-retention.py                      # the whole registry, JSON on stdout
  fleet-retention.py --only 'benchmarks/aime' 'agents/*'   # a glob subset
  fleet-retention.py --fresh              # discard resumable state first

Env: REGISTRY (default ghcr.io/exgentic), GH_TOKEN / GITHUB_TOKEN,
     RETENTION_JOBS (REST concurrency, default 8), RETENTION_REGISTRY_JOBS
     (default 32), RETENTION_DAYS (default 90), RETENTION_FLOOR (rate-limit
     headroom to leave, default 200), RETENTION_STATE (resume dir),
     RETENTION_NOW (freeze "now", RFC3339 — tests),
     RETENTION_API_BASE / RETENTION_REGISTRY_BASE (redirect hosts — tests).
Exit: 0 the census is complete, 3 partially enumerated (re-run to continue),
      non-zero on a failure that would make the census lie.

Python, like fleet-index.py next door and for the same reason: a few thousand
paginated HTTP calls whose failures must be told apart from empty answers, plus
rate-limit-window arithmetic, is what a shell script does badly.
"""

from __future__ import annotations

import argparse
import base64
import fnmatch
import json
import os
import random
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import UTC, datetime, timedelta

REGISTRY = os.environ.get("REGISTRY", "ghcr.io/exgentic")
HOST, _, ORG = REGISTRY.partition("/")
API = os.environ.get("RETENTION_API_BASE", "https://api.github.com")
REG = os.environ.get("RETENTION_REGISTRY_BASE", f"https://{HOST}")
JOBS = int(os.environ.get("RETENTION_JOBS", "8"))
REGISTRY_JOBS = int(os.environ.get("RETENTION_REGISTRY_JOBS", "32"))
DAYS = int(os.environ.get("RETENTION_DAYS", "90"))
# Headroom left on the shared 5,000/hr budget so a sweep never starves the
# release pipeline or another automation holding the same token.
FLOOR = int(os.environ.get("RETENTION_FLOOR", "200"))
# The list endpoint rejects per_page * page > 10000, so 100 pages of 100 is
# every package it will ever return, whatever the org actually holds.
API_CEILING = 10000
PER_PAGE = 100
# One registry token carries many repository scopes: ~10k packages cost ~100
# token calls, not 10k (fleet-index.py's measurement).
SCOPES_PER_TOKEN = 100
# A manifest list's children. Attestation entries ride at unknown/unknown and
# are just as reachable as a per-arch child — deleting one breaks provenance on
# a live image, so this Accept set asks for indexes AND plain manifests.
MANIFEST_ACCEPT = (
    "application/vnd.oci.image.index.v1+json, "
    "application/vnd.docker.distribution.manifest.list.v2+json, "
    "application/vnd.oci.image.manifest.v1+json, "
    "application/vnd.docker.distribution.manifest.v2+json"
)
CACHE_PREFIX = "buildcache/"
CLOCK_NOTE = (
    "created_at is NOT the moment a digest stopped being referenced; the API "
    "records no such field. A version listed `aged` may still be retained by "
    "rule 22 — it could have held `latest` until yesterday. This is a census "
    "for review, NOT an authorisation to delete."
)
TRUNCATED_NOTE = (
    f"The packages list endpoint rejects per_page*page > {API_CEILING}, so at "
    "most that many packages are reachable through it. This sweep saw every "
    "package the endpoint would return and CANNOT prove that is all of them."
)


def die(msg: str) -> str:
    sys.exit(f"fleet-retention: {msg}")


def now() -> datetime:
    """Frozen by RETENTION_NOW so a test asserting a 90-day boundary is not a
    time bomb."""
    if fixed := os.environ.get("RETENTION_NOW"):
        return datetime.fromisoformat(fixed.replace("Z", "+00:00"))
    return datetime.now(UTC)


def parse_time(stamp: str) -> datetime:
    return datetime.fromisoformat(stamp.replace("Z", "+00:00"))


def gh_token() -> str:
    for var in ("GH_TOKEN", "GITHUB_TOKEN"):
        if os.environ.get(var):
            return os.environ[var]
    out = subprocess.run(
        ["gh", "auth", "token"], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        die("no GH_TOKEN, and `gh auth token` failed")
    return out.stdout.strip()


# ── HTTP: the two budgets ────────────────────────────────────────────────────
# One lock serialises the rate-limit *wait*, not the requests: when the budget
# runs low every worker must park until the window resets, and each deciding on
# its own turned one sleep into JOBS staggered ones.
_pause = threading.Lock()


def _throttle(headers) -> None:
    """Park the pool when the shared budget is nearly spent.

    A sweep that spends the last of the hour's requests takes the release
    pipeline down with it, so it stops at FLOOR and waits for the window rather
    than racing to exhaustion.
    """
    remaining = headers.get("X-RateLimit-Remaining")
    reset = headers.get("X-RateLimit-Reset")
    if remaining is None or reset is None or int(remaining) >= FLOOR:
        return
    with _pause:
        wait = int(reset) - int(time.time()) + 1
        if wait <= 0:
            return
        print(
            f"fleet-retention: {remaining} requests left; waiting {wait}s for the "
            "rate-limit window",
            file=sys.stderr,
        )
        time.sleep(wait)


def request(url: str, headers: dict[str, str], throttle: bool = False) -> tuple:
    """`(body, response-headers)`, retrying only what is worth retrying.

    A 404 is an answer. A 5xx, a dropped connection or a rate-limit refusal is
    not — GHCR drops enough of them that treating one as an answer is how a
    sweep comes back smaller than the fleet (fleet-status.sh records two sweeps
    minutes apart disagreeing by six images). Retries are bounded; exhausting
    them raises, and the caller decides whether that poisons a package.
    """
    req = urllib.request.Request(url, headers=headers)
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                body = json.load(r)
                if throttle:
                    _throttle(r.headers)
                return body, r.headers
        except urllib.error.HTTPError as e:
            # 403/429 with a Retry-After is the secondary rate limit: obey it
            # verbatim instead of hammering, which is what earns a longer block.
            if e.code in (403, 429) and e.headers.get("Retry-After"):
                time.sleep(int(e.headers["Retry-After"]) + 1)
                continue
            if e.code in (401, 403, 404, 400) or attempt == 3:
                raise
        except (OSError, json.JSONDecodeError):
            if attempt == 3:
                raise
        time.sleep(2**attempt + random.random())
    raise AssertionError  # unreachable


def registry_token(repos: list[str], gh: str) -> str:
    basic = base64.b64encode(f"x:{gh}".encode()).decode()  # GHCR ignores the user
    scopes = "&".join(
        f"scope={urllib.parse.quote(f'repository:{ORG}/{r}:pull', safe='')}"
        for r in repos
    )
    body, _ = request(
        f"{REG}/token?service={HOST}&{scopes}", {"Authorization": f"Basic {basic}"}
    )
    return body["token"]


# ── enumeration: what the registry holds ─────────────────────────────────────


def packages(gh: str) -> tuple[list[str], bool]:
    """Every container package the list endpoint will return, and whether it
    truncated.

    Only packages attributed to this repository: the org's other repos publish
    into the same namespace, and their retention is not ours to census. Rows
    without a `repository` key at all are kept — dropping them would silently
    shrink the census on a field the API does not promise.
    """
    headers = {
        "Authorization": f"Bearer {gh}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    names: list[str] = []
    page = 1
    truncated = False
    while page * PER_PAGE <= API_CEILING:
        url = (
            f"{API}/orgs/{ORG}/packages?package_type=container"
            f"&per_page={PER_PAGE}&page={page}"
        )
        try:
            body, _ = request(url, headers, throttle=True)
        except urllib.error.HTTPError as e:
            # The ceiling is enforced with an error, not an empty page: measured
            # against this org, page 100 answers 500 and page 101 a 400 naming
            # the limit. But a 500 is also what a genuinely broken endpoint
            # returns, and calling that "truncated" would report a capped census
            # as complete-but-capped when the sweep simply failed (rule 14, fail
            # dirty). So only an error at the very edge of the window counts as
            # the ceiling; anywhere earlier is a failure and propagates.
            at_ceiling = (page + 1) * PER_PAGE > API_CEILING
            if at_ceiling and e.code in (400, 422, 500):
                truncated = True
                break
            raise
        if not isinstance(body, list) or not body:
            break
        for pkg in body:
            repo = (pkg.get("repository") or {}).get("name")
            if repo in (None, "eval-containers"):
                names.append(pkg["name"])
        if len(body) < PER_PAGE:
            break
        page += 1
    else:
        # Left the loop by the ceiling with a full last page: there is more the
        # endpoint will not serve.
        truncated = True
    return sorted(set(names)), truncated


def versions(pkg: str, gh: str) -> list[dict]:
    """Every version of one package, paginated.

    Packages routinely exceed one page (aime holds 167, buildcache/bases-amd64
    337), and a sweep that read only the first page would call the rest
    unreachable and nominate live digests for deletion.
    """
    headers = {
        "Authorization": f"Bearer {gh}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    name = urllib.parse.quote(pkg, safe="")
    out: list[dict] = []
    page = 1
    while True:
        body, _ = request(
            f"{API}/orgs/{ORG}/packages/container/{name}/versions"
            f"?per_page={PER_PAGE}&page={page}",
            headers,
            throttle=True,
        )
        if not isinstance(body, list) or not body:
            break
        out += body
        if len(body) < PER_PAGE:
            break
        page += 1
    return out


def children(pkg: str, digest: str, bearer: str) -> list[str]:
    """The digests a manifest references, or [] when it references none.

    Read by DIGEST, never by tag: reading `:latest` races the nightly publish,
    and expanding the new index against an older version list would orphan the
    previous index's children into the candidate set — the exact live-digest
    deletion this tool exists to prevent.
    """
    body, _ = request(
        f"{REG}/v2/{ORG}/{pkg}/manifests/{digest}",
        {"Authorization": f"Bearer {bearer}", "Accept": MANIFEST_ACCEPT},
    )
    # Every entry, attestations at unknown/unknown included: an attestation
    # manifest is as reachable as a per-arch one. fleet-status.sh filters those
    # out to count platforms, which is right there and would be fatal here.
    return [m["digest"] for m in body.get("manifests") or [] if m.get("digest")]


# ── the verdict ──────────────────────────────────────────────────────────────


def classify(pkg: str, vs: list[dict], bearer: str, cutoff: datetime) -> dict:
    """One package's census: a verdict per version, and a roll-up.

    Reachability is computed before any age is considered, because age is only
    ever a tiebreak among things nothing points at.
    """
    if pkg.startswith(CACHE_PREFIX):
        # Build cache, not a published digest: rule 22's subject does not reach
        # it, and nothing else authorises deleting it either. Counted, never a
        # candidate. A shorter clock for cache needs a rule that says so.
        return {
            "verdict": "cache",
            "versions": len(vs),
            "reachable": 0,
            "counts": {"cache": len(vs)},
            "aged": [],
        }

    tagged = [v for v in vs if v["metadata"]["container"]["tags"]]
    reachable = {v["name"] for v in tagged}
    unreadable = []
    for v in tagged:
        try:
            reachable.update(children(pkg, v["name"], bearer))
        except (urllib.error.HTTPError, OSError, json.JSONDecodeError) as e:
            # A tagged manifest we could not read has UNKNOWN children, so the
            # reachable set is incomplete and every candidate in this package is
            # suspect. Withhold the whole package rather than nominate a digest
            # that a manifest we never read may point at (rule 14, fail dirty).
            unreadable.append({"digest": v["name"], "error": str(e)})

    counts = {"tagged": len(tagged), "child": 0, "recent": 0, "aged": 0}
    aged = []
    for v in vs:
        if v["name"] in reachable:
            if not v["metadata"]["container"]["tags"]:
                counts["child"] += 1
            continue
        created = parse_time(v["created_at"])
        if created >= cutoff:
            counts["recent"] += 1
            continue
        counts["aged"] += 1
        aged.append(
            {
                "id": v["id"],
                "digest": v["name"],
                "created_at": v["created_at"],
                "age_days": (now() - created).days,
                "because": f"nothing tagged in {pkg} resolves to it, and it "
                f"was created {(now() - created).days}d ago "
                "(a proxy for supersession, not proof of it)",
            }
        )

    if unreadable:
        # Named, not swallowed, and stripped of candidates: the census reports
        # that it could not answer for this package. The `aged` tally goes with
        # them — a withheld candidate must not be counted as one in the fleet
        # totals, or the summary reports review candidates that no package's
        # `aged` list actually holds. Those versions are `unreadable` now, which
        # is what "could not answer" means.
        # Everything the incomplete reachable set left unaccounted for — the
        # would-be candidates and the would-be `recent` — is `unreadable`: its
        # status is precisely what we could not determine.
        undetermined = counts.pop("aged") + counts.pop("recent")
        return {
            "verdict": "blocked",
            "versions": len(vs),
            "reachable": len(reachable),
            "counts": counts | {"unreadable": undetermined},
            "unreadable": unreadable,
            "aged": [],
        }

    if counts["aged"] and counts["aged"] == len(vs):
        # Every version a candidate means nothing is tagged at all — usually a
        # build whose per-arch images landed and whose manifest list was never
        # stitched, in which case the fix is fleet-tag.sh, not deletion. The
        # tags it does hold are reported so a reviewer can tell which it is.
        verdict = "would-empty"
    elif counts["aged"]:
        verdict = "aged"
    elif counts["recent"] or counts["child"]:
        verdict = "holds"
    else:
        verdict = "clean"
    out = {
        "verdict": verdict,
        "versions": len(vs),
        "reachable": len(reachable),
        "counts": counts,
        "aged": aged,
    }
    if verdict == "would-empty":
        out["all_tags"] = sorted(
            {t for v in vs for t in v["metadata"]["container"]["tags"]}
        )
    return out


# ── resumable state ──────────────────────────────────────────────────────────


def state_dir(explicit: str | None) -> str:
    if explicit:
        return explicit
    if env := os.environ.get("RETENTION_STATE"):
        return env
    cache = os.environ.get("XDG_CACHE_HOME") or os.path.expanduser("~/.cache")
    return os.path.join(cache, "eval-containers", "retention")


def state_path(root: str, pkg: str) -> str:
    return os.path.join(root, urllib.parse.quote(pkg, safe="") + ".json")


# ── main ─────────────────────────────────────────────────────────────────────


def sweep(pkgs: list[str], gh: str, root: str) -> None:
    """Census every package that has no result yet, writing each as it lands.

    A whole-registry sweep is ~1.5M versions over ~20k rate-limited pages —
    hours, spanning several rate-limit windows — so progress is durable per
    package and a re-run continues rather than restarting.
    """
    todo = [p for p in pkgs if not os.path.exists(state_path(root, p))]
    if not todo:
        return
    cutoff = now() - timedelta(days=DAYS)
    batches = [
        todo[i : i + SCOPES_PER_TOKEN] for i in range(0, len(todo), SCOPES_PER_TOKEN)
    ]
    done = 0
    lock = threading.Lock()

    def one(pkg: str, bearer: str) -> None:
        nonlocal done
        try:
            result = classify(pkg, versions(pkg, gh), bearer, cutoff)
        except urllib.error.HTTPError as e:
            if e.code == 404:
                # Listed, then gone: a package deleted mid-sweep is not an
                # error, it is simply absent from the census.
                result = {"verdict": "absent", "versions": 0, "aged": []}
            else:
                raise
        tmp = state_path(root, pkg) + ".part"
        with open(tmp, "w") as f:
            json.dump(result, f)
        os.replace(tmp, state_path(root, pkg))  # atomic: no half-written result
        with lock:
            done += 1
            if done % 250 == 0:
                print(
                    f"fleet-retention: {done}/{len(todo)} packages",
                    file=sys.stderr,
                )

    with ThreadPoolExecutor(max_workers=REGISTRY_JOBS) as pool:
        bearers = list(pool.map(lambda b: registry_token(b, gh), batches))
    work = [
        (p, bearer)
        for batch, bearer in zip(batches, bearers, strict=True)
        for p in batch
    ]
    with ThreadPoolExecutor(max_workers=JOBS) as pool:
        # list() so an exception in any worker propagates instead of being
        # dropped with the generator.
        list(pool.map(lambda pb: one(*pb), work))


def main() -> None:
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument(
        "--only",
        nargs="+",
        metavar="GLOB",
        help="census only packages matching these globs (e.g. 'benchmarks/*')",
    )
    ap.add_argument("--state", help="resume directory (default: XDG cache)")
    ap.add_argument(
        "--fresh", action="store_true", help="discard resumable state first"
    )
    args = ap.parse_args()

    gh = gh_token()
    root = state_dir(args.state)
    os.makedirs(root, exist_ok=True)
    listing = os.path.join(root, "packages.json")

    if args.fresh:
        for f in os.listdir(root):
            os.remove(os.path.join(root, f))

    # The universe is frozen on the first run and reused by every resume: a
    # re-listing would shift under a sweep the nightly publish is adding to,
    # and a census assembled from two different universes is not a census.
    if os.path.exists(listing):
        with open(listing) as f:
            saved = json.load(f)
        names, truncated = saved["packages"], saved["truncated"]
    else:
        names, truncated = packages(gh)
        if not names:
            die("the packages endpoint returned nothing — refusing an empty census")
        with open(listing, "w") as f:
            json.dump({"packages": names, "truncated": truncated}, f)

    selected = (
        [n for n in names if any(fnmatch.fnmatch(n, g) for g in args.only)]
        if args.only
        else names
    )
    if not selected:
        die(f"no package matches {args.only}")

    sweep(selected, gh, root)

    results = {}
    missing = 0
    for pkg in selected:
        path = state_path(root, pkg)
        if not os.path.exists(path):
            missing += 1
            continue
        with open(path) as f:
            results[pkg] = json.load(f)
    if missing:
        # A partial census whose reachable sets are incomplete is the one
        # artifact never to emit: its candidate list would be reviewed as if
        # complete. Say what is left and exit distinctly.
        print(
            f"fleet-retention: {len(results)} of {len(selected)} packages "
            "enumerated; re-run to continue",
            file=sys.stderr,
        )
        sys.exit(3)

    totals = {
        k: 0 for k in ("tagged", "child", "recent", "aged", "cache", "unreadable")
    }
    versions_seen = 0
    for r in results.values():
        versions_seen += r.get("versions", 0)
        for k, v in (r.get("counts") or {}).items():
            totals[k] = totals.get(k, 0) + v
    would_empty = sorted(p for p, r in results.items() if r["verdict"] == "would-empty")
    blocked = sorted(p for p, r in results.items() if r["verdict"] == "blocked")

    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
        cwd=os.path.dirname(os.path.abspath(__file__)),
    ).stdout.strip()

    json.dump(
        {
            "source": {"repo": "Exgentic/eval-containers", "commit": commit},
            "generated": now().strftime("%Y-%m-%dT%H:%M:%SZ"),
            "registry": REGISTRY,
            "rule": "delivery/RULES.md:22",
            "retention_days": DAYS,
            "clock": {"basis": "created_at", "sound": False, "note": CLOCK_NOTE},
            "completeness": {
                "packages_listed": len(names),
                "packages_censused": len(results),
                "api_ceiling": API_CEILING,
                "truncated": truncated,
                "note": TRUNCATED_NOTE,
            },
            "totals": {"versions": versions_seen} | totals,
            "would_empty": would_empty,
            "blocked": blocked,
            "packages": results,
        },
        sys.stdout,
        indent=1,
    )
    sys.stdout.write("\n")

    print(
        f"retention: {len(results)} packages, {versions_seen} versions — "
        f"{totals['tagged']} tagged, {totals['child']} child, "
        f"{totals['recent']} recent, {totals['aged']} aged, "
        f"{totals['cache']} cache, {totals['unreadable']} unreadable"
        + (f"; {len(would_empty)} would-empty" if would_empty else "")
        + (f"; {len(blocked)} blocked" if blocked else "")
        + ("; LIST TRUNCATED" if truncated else ""),
        file=sys.stderr,
    )
    if totals["aged"]:
        print(
            f"retention: {totals['aged']} version(s) are review candidates — "
            "an unsound proxy for rule 22, not a clearance to delete",
            file=sys.stderr,
        )


if __name__ == "__main__":
    main()
