# Running Dock Tests Locally

**Status:** Practical guide
**Date:** April 2026

This document is the practical counterpart to [RULES.md](RULES.md). RULES defines what tests MUST do; this doc explains how to run them on your machine without drowning in disk usage or OOMing your VM.

## Principle

**Test what you touched locally. Let CI test everything.**

Dock has 96+ benchmarks and 17+ agents. That's 1600+ possible eval combinations, most of which you'll never need locally. Build only what you're working on; pull everything else from the registry.

## What runtime you need

Dock is a **Docker-first** project. Everything — Dockerfiles, compose files, CI — is written against the standard Docker API. You can run it on any OCI-compatible runtime that exposes a Docker-compatible socket:

- **Docker Desktop** — the canonical path, what CI uses. Easiest if you're on Mac or Windows and don't have a strong preference.
- **Docker Engine** (Linux) — what the release pipeline runs against. Identical to Docker Desktop for our purposes.
- **Podman** with the `docker` compatibility CLI — works if you already have Podman installed. A few setup gotchas below.
- **Colima / OrbStack / Rancher Desktop** — also work; same Docker-compatible API.

**You interact with Docker through the `docker` command and nothing else.** `docker build`, `docker compose`, `docker buildx bake`. The underlying engine doesn't matter. If you find yourself typing `podman` directly, you're off the happy path — fix your setup and use `docker` instead.

## Disk Budget

| Artifact | Typical size | How many |
|----------|-------------|----------|
| Benchmark image | 500 MB – 2 GB | 96 |
| Agent image | 500 MB – 1 GB | 17 |
| Eval combination | 1 – 3 GB | on demand |
| Per-task benchmark (swe-bench, compilebench) | 2 GB × N tasks | 500+ |

Building everything locally is expensive — the full fleet is ~150 GB of images before cleanup and ~30–90 min of build time depending on parallelism. It's technically fine on Mac with Rosetta (see Level 2b below for the parallel sweep flag), but prefer the targeted Level 2 workflow for day-to-day dev. CI builds the fleet on every release.

## Setup: Docker Desktop (recommended)

### 1. Install

Download and install Docker Desktop: https://www.docker.com/products/docker-desktop/

### 2. Size the VM

Docker Desktop → Settings → Resources. Give it half your RAM and half your cores (e.g. 32 GB / 10 CPUs on a 64 GB machine). Same budget as Podman.

### 3. Enable Rosetta (Apple Silicon only) — REQUIRED

Most benchmarks are `linux/amd64`. Without Rosetta, Docker Desktop falls back to QEMU, which is **~10× slower** and often crashes on Python extensions (pyarrow segfaults, numpy SIGILL, etc.).

Docker Desktop → Settings → General → **"Use Rosetta for x86_64/amd64 emulation on Apple Silicon"** (checkbox). Apply & restart.

Verify it's actually active by running an amd64 image:

```bash
docker run --rm --platform=linux/amd64 python:3.12-slim python -c "import platform; print(platform.machine())"
# Should print: x86_64
```

If builds of python-heavy benchmarks segfault or SIGILL on first use, Rosetta isn't on — re-check the setting.

### 4. Enable BuildKit garbage collection

Docker Desktop → Settings → Builders → edit the default builder → set **"Garbage collection"** to a fixed budget (e.g. 20 GB). BuildKit reclaims automatically when it crosses the threshold. Without this, `docker build` cache grows unbounded.

## Setup: Podman (alternative)

If you already use Podman, it works — Dock's Dockerfiles and compose files use vanilla syntax with no Docker-only or Podman-only features. You just need to install the Docker-compat CLI shim and point it at Podman, then use `docker` commands from there.

```bash
brew install docker                    # the docker CLI, client only
podman machine init                    # if you don't already have one
podman machine set --memory 32768 --cpus 10
```

Enable Rosetta on the machine (REQUIRED on Apple Silicon):

```bash
podman machine ssh "sudo touch /etc/containers/enable-rosetta"
podman machine stop && podman machine start
```

Point the `docker` CLI at Podman's socket and verify:

```bash
export DOCKER_HOST="unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
docker version     # should report a running server
docker info        # should say Context: default, Server OS: linux
```

From here, use **`docker` commands for everything** — `docker build`, `docker compose`, etc. Never invoke `podman` directly for Dock workflows. If you need BuildKit garbage collection, set it via the podman machine:

```bash
podman machine ssh <<'EOF'
sudo tee /etc/containers/containers.conf.d/gc.conf <<CONF
[build]
gc_enabled = true
gc_keep_storage = "20GB"
CONF
EOF
podman machine stop && podman machine start
```

Note: Podman's docker-compat socket does not support `buildx`. For fleet builds (`docker buildx bake`), use real Docker. Single-image dev-loop builds (`docker build benchmarks/aime/`) work on Podman-backed `docker`.

## Test Levels

### Level 1: Structural validation (seconds)

No containers built. Catches missing Dockerfiles, missing labels, broken compose files.

```bash
cargo test --test check structural_validation                      # every benchmark + agent on disk
cargo test --test compose -- --ignored       # cargo equivalent for the 29 committed benchmarks
```

Run on every commit.

### Level 2: Build the thing you touched

Local dev loop: build exactly the benchmark or agent you're working on. Nothing more.

```bash
# One benchmark
docker build -t local/aime benchmarks/aime/

# One agent
docker build -t local/claude-code agents/claude-code/

# One eval combination (benchmark + agent + model)
dock build eval aime --agent codex
```

That's it. Don't try to build the fleet locally — CI does that via [RELEASE.md](../RELEASE.md).

### Level 2b: Full-fleet build sweep (local)

Locally buildable and valid on Mac/Linux with Podman or Docker — the "don't build the fleet locally" warning above is about disk/time cost, not capability. With Rosetta on (see setup above), every image builds fine. The cost is just disk (~150 GB peak before `ImageGuard` cleans each tag) and time.

```bash
# Serial (default) — one image at a time, ~90 min for the full fleet
cargo test --test build -- --ignored

# Parallel — run up to N builds concurrently via DOCK_BUILD_PARALLEL
DOCK_BUILD_PARALLEL=4 cargo test --test build -- --ignored
```

`DOCK_BUILD_PARALLEL=N` bounds the number of in-flight `docker build` calls. Rule of thumb: `N ≈ VM_CPUS / 2` (BuildKit saturates a couple of cores per image during `RUN` layers). On a 5-cpu / 32 GB podman VM, `N=4` is a good fit and cuts the full sweep roughly to 1/3. Higher values (`N=6+`) mostly fight each other on the network during `apt-get update` / `pip install`.

The harness still verifies via testcontainers-rs — it only parallelizes the outer loop, not the per-image build mechanism (per `tests/RULES.md` rule 6b).

### Level 3: Replay tests (minutes, free)

Full pipeline with recorded LLM trajectories. Deterministic, zero API cost.

```bash
cargo test --test replay -- --ignored --test-threads=6
```

Rule of thumb: `--test-threads = VM_GB / 4` (each replay stack uses ~4 GB peak).

### Level 4: Recording fixtures (costs API calls)

One-time. Runs a real task with a real model, saves the trajectory as a fixture.

```bash
# Record one combination
TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-4.1-mini \
  docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit

cp output/aime/0/model/trajectory.jsonl \
   tests/fixtures/aime-0-codex.trajectory.jsonl
```

Use `gpt-4.1-mini` — cheapest model that works. One fixture per combination forever.

## Exploring What's Built

Three thin wrappers around Docker's native commands, so you see both the result and the underlying `docker` call:

```bash
# List dock images with sizes (wraps `docker images`)
dock images                     # all dock images
dock images benchmarks          # just benchmarks
dock images agents

# Inspect a dock image (wraps `docker inspect`)
dock inspect aime               # benchmark
dock inspect codex --category agents
```

## Reclaiming Disk

```bash
# Safe: prune build cache + dangling images
dock prune

# Destructive: wipe all dock.* labeled images
dock prune --all
```

With BuildKit GC configured in setup, you rarely need `dock prune` manually.

## Common Workflows

**Starting fresh on a benchmark:**
```bash
# 1. Structural smoke test
cargo test --test check structural_validation

# 2. Build + verify labels
docker build -t local/aime benchmarks/aime/

# 3. Run one task with a real model
TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-4.1-mini \
  docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit

# 4. Check the output
cat output/aime/0/task/result.json
```

**Before pushing a PR:**
```bash
cargo test --test check structural_validation                      # every benchmark + agent structurally
cargo test --test compose -- --ignored       # cargo compose tests
docker build benchmarks/aime/                # only the ones you changed
cargo test --test replay -- --ignored        # only the ones you changed
```

Everything else — full fleet build, registry push, multi-arch — is CI's job. See [RELEASE.md](../RELEASE.md).

**Reclaim a weekend's worth of builds:**
```bash
dock prune --all
```

## Per-Task Benchmarks

`swe-bench`, `compilebench`, `terminal-bench` use `ARG TASK_ID` at build time — each task is a separate image. **Never build them all.** Pick one:

```bash
dock build bench swe-bench --task-id sympy__sympy-24066
```

## Registry Caching (Future)

Once images are published to the registry, local testing becomes:

```bash
dock run aime --task-id 0 --agent codex --model gpt-5.4
```

No local builds needed. CI builds once; everyone pulls.

## References

- [Testing Policy](RULES.md) — normative spec
- [CLI](../src/RULES.md) — CLI design rules
- [Containers](containers/RULES.md) — container test rules
- [Release pipeline](../RELEASE.md) — how CI builds and pushes the fleet
