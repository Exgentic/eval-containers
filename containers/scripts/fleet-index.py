#!/usr/bin/env python3
"""fleet-index — an inventory of every artifact the fleet has published.

The dashboard's launch page and `eval-containers list` both need one fact the
repository cannot answer about itself: what is actually in the registry. The tree
over-answers it (a benchmark whose base fails to build publishes nothing), and so
does any model built by multiplying families by an agent lineup — that is how a
catalog came to offer 792 combos for the ~100 that exist, and how an agent added
to the lineup with one published benchmark (#516) became offerable against all of
them. So this enumerates: every artifact by name, with its tags, and the `eval.*`
labels of every component. Consumers search and filter the list; nobody
re-derives it.

Names come from the tree — every component, every benchmark x agent combo, every
baked task — and each is then proved against the registry, so a name is in the
index only if the registry answered for it. The registry cannot be listed
instead: `/v2/_catalog` returns all of GHCR with no way to seek to one org, and
the packages REST listing is capped at 10000 results, which this org already
exceeds.

Every candidate is asked about, every time: ~30000 of them in ~2 minutes. Testing
a few of a group before asking about the rest would be a third of the work and
would lie — `cline` publishes 3 per-task combos out of 664, so any sample small
enough to be worth taking misses it. The registry reads do not touch the GitHub
API budget (measured: 100 of them moved `core.remaining` by zero), so the cost of
being exact is wall-clock on a runner, once a cycle, for everyone.

Usage: fleet-index.py > index.json
Env: REGISTRY (default ghcr.io/exgentic), GH_TOKEN, CATALOG_JOBS (default 32),
     CATALOG_AGENTS (the agent lineup; defaults to every agent in the tree).

Self-contained on purpose: it needs no artifact from a release, so the same
script answers on a schedule — which is what catches a deleted image or a push
that happened outside a release.

Python, not bash like its neighbours: a few thousand parallel HTTP calls whose
failures must be told apart from empty answers is what a shell script does badly.
"""

from __future__ import annotations

import base64
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import UTC, datetime

REGISTRY = os.environ.get("REGISTRY", "ghcr.io/exgentic")
HOST, _, ORG = REGISTRY.partition("/")
JOBS = int(os.environ.get("CATALOG_JOBS", "64"))
# One token carries many repository scopes, so a token is minted per batch rather
# than per repository: 7k artifacts cost ~70 token calls, not 7k.
SCOPES_PER_TOKEN = 100
HERE = os.path.dirname(os.path.abspath(__file__))
CONTAINERS = os.path.normpath(os.path.join(HERE, ".."))
# Every kind the fleet publishes as an image of its own.
KINDS = ("core", "gateways", "models", "agents", "benchmarks")


def die(msg: str) -> None:
    sys.exit(f"fleet-index: {msg}")


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


def components(kind: str) -> list[str]:
    root = os.path.join(CONTAINERS, kind)
    if not os.path.isdir(root):
        return []
    return sorted(
        d
        for d in os.listdir(root)
        if not d.startswith("_") and os.path.isfile(os.path.join(root, d, "Dockerfile"))
    )


def per_task(name: str) -> bool:
    """A benchmark bakes one image per task iff it says so on a LABEL line — the
    single source of truth for per-task-ness (benchmarks/RULES.md 24f)."""
    path = os.path.join(CONTAINERS, "benchmarks", name, "Dockerfile")
    try:
        with open(path) as f:
            return any(
                line.lstrip().startswith("LABEL ")
                and 'eval.benchmark.env="per-task"' in line
                for line in f
            )
    except OSError:
        return False


def labels(kind: str, name: str) -> dict[str, str]:
    """The component's `eval.*` labels, off its Dockerfile.

    The tree, not the published image's config: at the commit this release
    publishes from they are the same labels, and reading them from the registry
    costs a manifest-index-config walk per component to catch the one case they
    differ — a component whose published image is older than the tree, which
    shows up in `images` as a missing artifact anyway.
    """
    path = os.path.join(CONTAINERS, kind, name, "Dockerfile")
    out: dict[str, str] = {}
    try:
        with open(path) as f:
            lines = f.readlines()
    except OSError:
        return out
    for line in lines:
        # On a LABEL line, so a comment or a RUN echo cannot invent one.
        rest = line.lstrip()
        if not rest.startswith("LABEL "):
            continue
        key, sep, value = rest[len("LABEL ") :].strip().partition('="')
        # A value the Dockerfile interpolates at build time (`${AGENT_VERSION}`)
        # is a placeholder here, not a fact worth filtering on.
        if (
            not sep
            or not value.endswith('"')
            or not key.startswith("eval.")
            or "${" in value
        ):
            continue
        out[key] = value[:-1]
    return dict(sorted(out.items()))


def task_ids(family: str) -> list[str]:
    """The task ids a per-task benchmark bakes.

    From its own tasks.txt when it has one — `#` headings and blanks skipped,
    since a heading read as a task id is an image nobody built — and otherwise
    from the `tasks/` directory of the upstream repo its build.sh pins, which is
    where terminal-bench, skills-bench and deepswe get theirs.
    """
    d = os.path.join(CONTAINERS, "benchmarks", family)
    listed = os.path.join(d, "tasks.txt")
    if os.path.isfile(listed):
        with open(listed) as f:
            return [
                t
                for t in (line.strip().lower() for line in f)
                if t and not t.startswith("#")
            ]
    build = os.path.join(d, "build.sh")
    if not os.path.isfile(build):
        return []
    with open(build) as f:
        text = f.read()
    repo = re.search(
        r"github\.com/([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+?)(?:\.git)?[\s\"'#]", text
    )
    ref = re.search(r"\b[0-9a-f]{40}\b", text)
    if not (repo and ref):
        # swe-bench-pro resolves its ids from a HuggingFace dataset and
        # swe-lancer from Docker Hub tags. Neither publishes per-task images
        # today, so there is nothing to miss; named, not swallowed.
        print(f"fleet-index: {family}: no enumerable task list", file=sys.stderr)
        return []
    out = subprocess.run(
        [
            "gh",
            "api",
            f"repos/{repo.group(1)}/contents/tasks?ref={ref.group(0)}",
            "--paginate",
            "--jq",
            '.[] | select(.type=="dir") | .name',
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if out.returncode != 0:
        die(f"{family}: upstream task list unreadable: {out.stderr.strip()[:200]}")
    return sorted({t.lower() for t in out.stdout.split() if t})


def candidates(tasks: dict[str, list[str]], agents: list[str]) -> list[str]:
    """Every artifact name this tree can publish. Proved against the registry
    below — naming one here claims nothing."""
    names = [f"{kind}/{name}" for kind in KINDS for name in components(kind)]
    for family in components("benchmarks"):
        if not os.path.isfile(
            os.path.join(CONTAINERS, "benchmarks", family, "compose.yaml")
        ):
            continue  # no compose.yaml, no combo image
        benchmarks = (
            [f"{family}-{t}" for t in tasks.get(family, ())]
            if per_task(family)
            else [family]
        )
        names += [f"benchmarks/{b}" for b in benchmarks if b != family]
        for b in benchmarks:
            names += [f"evals/{b}--{a}" for a in agents]
            if b == family:
                # The standalone bundle is a name suffix on the same pair
                # (src/RULES.md 11), published only for shared-env combos.
                names += [f"evals/{b}--{a}-standalone" for a in agents]
    return sorted(set(names))


def get_json(url: str, headers: dict[str, str]) -> dict:
    req = urllib.request.Request(url, headers=headers)
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            if e.code in (401, 403, 404) or attempt == 3:
                raise
        except OSError:
            if attempt == 3:
                raise
    raise AssertionError  # unreachable


def registry_token(repos: list[str], gh: str) -> str:
    basic = base64.b64encode(f"x:{gh}".encode()).decode()  # GHCR ignores the user
    scopes = "&".join(
        f"scope={urllib.parse.quote(f'repository:{ORG}/{r}:pull', safe='')}"
        for r in repos
    )
    return get_json(
        f"https://{HOST}/token?service={HOST}&{scopes}",
        {"Authorization": f"Basic {basic}"},
    )["token"]


def tags_of(repo: str, bearer: str) -> tuple[str, list[str]]:
    """The repo's tags, empty when the registry has never heard of it.

    A package can also exist with nothing tagged — a build that never published —
    and listing that as available is how a launch becomes an ImagePullBackOff, so
    it reads the same as absent. Anything else raises: a blip must not shrink the
    index (delivery/RULES.md 14, fail dirty).
    """
    try:
        body = get_json(
            f"https://{HOST}/v2/{ORG}/{repo}/tags/list",
            {"Authorization": f"Bearer {bearer}"},
        )
    except urllib.error.HTTPError as e:
        if e.code in (401, 403, 404):
            return repo, []
        raise
    return repo, sorted(body.get("tags") or [])


def sweep(repos: list[str], gh: str, fn):
    """`fn(repo, bearer)` over every repo, in one pool.

    A token carries a batch of repository scopes, but minting one batch's token
    and then waiting for that batch's slowest probe before minting the next made
    the sweep 300 sequential steps: the tokens are minted first, in parallel, and
    every probe then runs against the pool as one flat list.

    Every repo yields a row or the run dies — a sweep that answered for fewer
    than it asked is a broken sweep, not a smaller fleet.
    """
    batches = [
        repos[i : i + SCOPES_PER_TOKEN] for i in range(0, len(repos), SCOPES_PER_TOKEN)
    ]
    with ThreadPoolExecutor(max_workers=JOBS) as pool:
        bearers = list(pool.map(lambda b: registry_token(b, gh), batches))
        work = [
            (r, bearer)
            for batch, bearer in zip(batches, bearers, strict=True)
            for r in batch
        ]
        out = list(pool.map(lambda rb: fn(*rb), work))
    if len(out) != len(repos):
        die(f"sweep answered {len(out)} of {len(repos)}")
    return out


def main() -> None:
    agents = os.environ.get("CATALOG_AGENTS", "").split() or components("agents")
    tasks = {f: task_ids(f) for f in components("benchmarks") if per_task(f)}

    names = candidates(tasks, agents)
    images = {n: t for n, t in sweep(names, gh_token(), tags_of) if t}
    if not images:
        die(f"none of {len(names)} candidates is published — refusing an empty index")

    declared = {
        f"{kind}/{c}": labels(kind, c)
        for kind in ("benchmarks", "agents", "models")
        for c in components(kind)
    }
    # Not filtered to what `images` holds: a per-task benchmark publishes no
    # `benchmarks/<family>` image of its own — only `benchmarks/<family>-<task>`
    # — so keying its labels off that image dropped swe-bench, terminal-bench and
    # skills-bench from the labels entirely, and with them any consumer's way to
    # tell that `evals/swe-bench-astropy__astropy-12907--codex` is a task of
    # swe-bench rather than a benchmark of its own. Labels are what a component
    # declares; `images` is what exists. They answer different questions.
    declared = {r: v for r, v in declared.items() if v}

    commit = subprocess.run(
        ["git", "-C", CONTAINERS, "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    json.dump(
        {
            "source": {"repo": "Exgentic/eval-containers", "commit": commit},
            "generated": datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "registry": REGISTRY,
            "images": images,
            "labels": declared,
        },
        sys.stdout,
        indent=1,
    )
    sys.stdout.write("\n")
    kinds = {}
    for n in images:
        kinds[n.split("/", 1)[0]] = kinds.get(n.split("/", 1)[0], 0) + 1
    print(
        f"index: {len(images)} of {len(names)} candidates published "
        f"({', '.join(f'{k} {v}' for k, v in sorted(kinds.items()))}), "
        f"{len(declared)} labelled components",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
