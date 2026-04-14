# Running Dock Tests Locally

**Status:** Practical guide
**Date:** April 2026

This document is the practical counterpart to [RULES.md](RULES.md). RULES defines what tests MUST do; this doc explains how to run them on your machine without drowning in disk usage or OOMing your VM.

## Principle

**Test what you touched locally. Let CI test everything.**

Dock has 77+ benchmarks and 11+ agents. That's 800+ possible eval combinations, most of which you'll never need locally. Build only what you're working on; pull everything else from the registry.

## Disk Budget

| Artifact | Typical size | How many |
|----------|-------------|----------|
| Benchmark image | 500 MB – 2 GB | 77 |
| Agent image | 500 MB – 1 GB | 11 |
| Eval combination | 1 – 3 GB | on demand |
| Per-task benchmark (swe-bench, compilebench) | 2 GB × N tasks | 500+ |

Building everything locally is **not** an option. Don't try.

## Mac Setup (Podman)

### 1. Size the VM

Give Podman half your RAM, half your cores:

```bash
podman machine stop
podman machine set --memory 32768 --cpus 10  # 32 GB, 10 CPUs
podman machine start
```

### 2. Enable auto garbage collection

Set once — Podman reclaims disk automatically when it crosses the threshold:

```bash
podman machine ssh <<'EOF'
sudo tee /etc/containers/containers.conf.d/gc.conf <<CONF
[engine]
image_parallel_copies = 4

[build]
gc_enabled = true
gc_keep_storage = "20GB"
CONF
EOF
podman machine stop && podman machine start
```

### 3. Enable Rosetta (Apple Silicon only) — REQUIRED

Most benchmarks are `linux/amd64`. Without Rosetta, Podman falls back to QEMU, which is **~10× slower** and often crashes on Python extensions (pyarrow segfaults, numpy SIGILL, etc.).

```bash
podman machine ssh "sudo touch /etc/containers/enable-rosetta"
podman machine stop && podman machine start
```

**Verify it's actually active:**

```bash
# Should print "Rosetta" — not "qemu"
podman machine ssh "cat /proc/sys/fs/binfmt_misc/rosetta 2>/dev/null | head -1 || echo 'NOT ACTIVE'"
```

If it says `NOT ACTIVE`, the trigger file didn't take effect — re-run the `touch` and restart the machine. This is the single biggest footgun on Apple Silicon: things appear to work, then silently crash on specific benchmarks.

## Test Levels

### Level 1: Compose validation (seconds)

Fast. No containers built. Catches YAML errors, bad extends, missing env vars.

```bash
cargo test --test compose -- --ignored
```

Run this on every commit.

### Level 2: Build the thing you touched

Local dev loop: build exactly the benchmark or eval you're working on. Nothing more.

```bash
# One benchmark
docker build -t local/aime benchmarks/aime/

# One eval combination (benchmark + agent + model)
dock build eval aime --agent codex
```

That's it. Don't try to build the fleet locally — CI does that via [RELEASE.md](../RELEASE.md).

### Level 3: Replay tests (minutes, free)

Full pipeline with recorded LLM trajectories. Deterministic, zero API cost.

```bash
# Requires DOCKER_HOST to point to podman socket
export DOCKER_HOST="unix:///var/folders/.../T/podman/podman-machine-default-api.sock"

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

With auto-GC enabled (see setup above), you rarely need `dock prune` manually.

## Common Workflows

**Starting fresh on a benchmark:**
```bash
# 1. Smoke test compose
cargo test --test compose -- --ignored compose_test::aime

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
cargo test --test compose -- --ignored       # all 77, <1s
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
