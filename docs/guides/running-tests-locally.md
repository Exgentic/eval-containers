# Running Eval Containers Tests Locally

**Status:** Practical guide
**Date:** April 2026

This document is the practical counterpart to [RULES.md](../../.agents/verification/RULES.md). RULES defines what tests MUST do; this doc explains how to run them on your machine without drowning in disk usage or OOMing your VM.

## Principle

**Test what you touched locally. Let CI test everything.**

Eval Containers has 96+ benchmarks and 17+ agents. That's 1600+ possible eval combinations, most of which you'll never need locally. Build only what you're working on; pull everything else from the registry.

## What runtime you need

Eval Containers is a **Docker-first** project. Everything — Dockerfiles, compose files, CI — is written against the standard Docker API. You can run it on any OCI-compatible runtime that exposes a Docker-compatible socket:

- **Docker Desktop** — the canonical path, what CI uses. Easiest if you're on Mac or Windows and don't have a strong preference.
- **Docker Engine** (Linux) — what the release pipeline runs against. Identical to Docker Desktop for our purposes.
- **Podman** with the `docker` compatibility CLI — works if you already have Podman installed; the Apple-Silicon setup has a few gotchas, all collected in [Run with Podman on Apple Silicon](podman-on-apple-silicon.md).
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

Podman works — the Dockerfiles and compose files are vanilla, with no Docker- or
Podman-only features — but the Apple-Silicon setup has several gotchas (Rosetta
machine-image pinning, the compose plugin, `DOCKER_HOST` for both the CLI **and**
the test harness, Ryuk under podman). They're all collected in one place:

**→ [Running with Podman on Apple Silicon](podman-on-apple-silicon.md)**

Once set up, use `docker` for everything — never invoke `podman` directly.

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
docker build -t local/aime containers/benchmarks/aime/

# One agent
docker build -t local/claude-code agents/claude-code/

# One eval combination (benchmark + agent + model)
eval-containers build eval aime --agent codex
```

That's it. Don't try to build the fleet locally — CI does that via [release pipeline](../../.agents/delivery/release/SKILL.md).

### Level 2b: Full-fleet build sweep (local)

Locally buildable and valid on Mac/Linux with Podman or Docker — the "don't build the fleet locally" warning above is about disk/time cost, not capability. With Rosetta on (see setup above), every image builds fine. The cost is just disk (~150 GB peak before `ImageGuard` cleans each tag) and time.

```bash
# Serial (default) — one image at a time, ~90 min for the full fleet
cargo test --test build -- --ignored

# Parallel — run up to N builds concurrently via EVAL_BUILD_PARALLEL
EVAL_BUILD_PARALLEL=4 cargo test --test build -- --ignored
```

`EVAL_BUILD_PARALLEL=N` bounds the number of in-flight `docker build` calls. Rule of thumb: `N ≈ VM_CPUS / 2` (BuildKit saturates a couple of cores per image during `RUN` layers). On a 5-cpu / 32 GB podman VM, `N=4` is a good fit and cuts the full sweep roughly to 1/3. Higher values (`N=6+`) mostly fight each other on the network during `apt-get update` / `pip install`.

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
# Record one combination — uses the shared `output` named volume from
# compose/services.yaml (the runner writes to /output inside the container).
EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f containers/benchmarks/aime/compose.yaml up --abort-on-container-exit

# The output lives in the named volume, not on the host filesystem.
# Extract via a one-shot alpine container that mounts it read-only.
docker run --rm -v aime_output:/output:ro alpine \
  cat /output/traces.jsonl > tests/run/replay/fixtures/aime-0-codex.traces.jsonl
```

The volume name follows `<benchmark>_output` (compose project + the `output` declared in `compose/services.yaml`). Sanity-check the result:
```bash
docker run --rm -v aime_output:/output:ro alpine cat /output/task/result.json
```

Use `gpt-5.4` — the cheap-but-capable default. One fixture per combination forever.

## Exploring What's Built

Three thin wrappers around Docker's native commands, so you see both the result and the underlying `docker` call:

```bash
# List eval-containers images with sizes (wraps `docker images`)
eval-containers images                     # all eval-containers images
eval-containers images benchmarks          # just benchmarks
eval-containers images agents

# Inspect a eval-containers image (wraps `docker inspect`)
eval-containers inspect aime               # benchmark
eval-containers inspect codex --category agents
```

## Reclaiming Disk

```bash
# Safe: prune build cache + dangling images
eval-containers prune

# Destructive: wipe all eval-containers.* labeled images
eval-containers prune --all
```

With BuildKit GC configured in setup, you rarely need `eval-containers prune` manually.

## Common Workflows

**Starting fresh on a benchmark:**
```bash
# 1. Structural smoke test
cargo test --test check structural_validation

# 2. Build + verify labels
docker build -t local/aime containers/benchmarks/aime/

# 3. Run one task with a real model
TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-4.1-mini \
  docker compose -f containers/benchmarks/aime/compose.yaml up --abort-on-container-exit

# 4. Check the output
cat output/aime/0/task/result.json
```

**Before pushing a PR:**
```bash
cargo test --test check structural_validation                      # every benchmark + agent structurally
cargo test --test compose -- --ignored       # cargo compose tests
docker build containers/benchmarks/aime/                # only the ones you changed
cargo test --test replay -- --ignored        # only the ones you changed
```

Everything else — full fleet build, registry push, multi-arch — is CI's job. See [release pipeline](../../.agents/delivery/release/SKILL.md).

**Reclaim a weekend's worth of builds:**
```bash
eval-containers prune --all
```

## Per-Task Benchmarks

`swe-bench`, `compilebench`, `terminal-bench` use `ARG TASK_ID` at build time — each task is a separate image. **Never build them all.** Pick one:

```bash
eval-containers build bench swe-bench --task-id sympy__sympy-24066
```

## Registry Caching (Future)

Once images are published to the registry, local testing becomes:

```bash
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4
```

No local builds needed. CI builds once; everyone pulls.

## References

- [Testing Policy](../../.agents/verification/RULES.md) — normative spec
- [CLI](../../.agents/src/RULES.md) — CLI design rules
- [Release pipeline](../../.agents/delivery/release/SKILL.md) — how CI builds and pushes the fleet
