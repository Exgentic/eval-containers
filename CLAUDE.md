# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

Eval Containers is a build system that produces Docker images and Compose files for AI agent evaluations. Every evaluation is one **benchmark** + one **agent** + one **model** — three independent axes, each a swappable Docker image. The `eval-containers` CLI (Rust) is a thin wrapper around `docker` and `docker compose`; everything also works without it.

## Commands

### Rust CLI (build, lint, test)

```bash
cargo build                        # build the CLI
cargo fmt --check                  # check formatting
cargo clippy -- -D warnings        # lint
cargo test                         # fast sanity suite (no Docker, no API keys)
cargo test --test check            # structural validation only
cargo test --test check structural # single gate within a test file
```

### Test levels (require --ignored to run)

```bash
# Level 1: Fast structural checks (seconds, no Docker)
cargo test --test check
cargo test --test compose -- --ignored

# Level 2: Build a specific image you touched
docker build -t local/aime benchmarks/aime/
docker build -t local/claude-code agents/claude-code/
eval-containers build eval aime --agent codex

# Level 3: Full fleet build (slow — ~90 min)
cargo test --test build -- --ignored
EVAL_BUILD_PARALLEL=4 cargo test --test build -- --ignored

# Level 4: Replay tests (no API keys needed)
cargo test --test replay -- --ignored --test-threads=6

# Level 5: Live tests (require API keys, release only)
cargo test --test live -- --ignored
```

### Running an evaluation

```bash
# Via CLI
eval-containers run --benchmark aime --task-id 0 --agent codex --model gpt-5.4

# Via plain docker compose (local dev)
EVAL_BENCHMARK=aime EVAL_TASK_ID=0 EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 \
  docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit

# Results
cat output/aime/0/task/result.json
```

### Disk management

```bash
eval-containers prune          # build cache + dangling images
eval-containers prune --all    # all eval-containers.* labeled images
```

## Architecture

### Three-image evaluation stack

Every evaluation is three Docker containers orchestrated by Compose:

1. **Benchmark image** (`benchmarks/<name>/`) — contains task data, grading logic (`/tests/test.sh`), and the shared `eval-entrypoint.sh`. Sets `TASK` and `EXPECTED_ANSWER` env vars for the agent.
2. **Agent image** (`agents/<name>/`) — packages an AI system with `/opt/agent/install.sh` and `/opt/agent/entrypoint.sh`. Reads `$TASK`, prints answer to stdout.
3. **Model image** (`models/<name>/`) — a pre-configured LiteLLM proxy that routes LLM calls, logs every request/response to `/output/model/trajectory.jsonl`, and enforces a spend cap. Holds the only copy of API keys.

The **eval image** (what actually runs) is a build-time combination: benchmark base layer + agent installed on top via `core/combination.Dockerfile`. The agent's `install.sh` runs at combine time; the combined image is what gets tagged and pushed.

### Execution flow (inside the eval container)

`/entrypoint.sh` (benchmark-specific) → `eval-materialize-task` (extracts task files) → `eval-entrypoint.sh` (shared core) → runs agent as non-root user → runs `test.sh` → writes `result.json`.

The shared entrypoint at `core/entrypoint/eval-entrypoint.sh` implements the two-knob versioning contract: container tag selects *which image* to pull; `EVAL_BENCHMARK_VERSION` / `EVAL_AGENT_VERSION` env vars select *what runs inside* (triggering `/eval-refetch-data` or `/eval-reinstall-agent` hooks if present).

### Output layout

```
output/{benchmark}/{task-id}/
  task/result.json      # { task_id, benchmark, reward, passed }
  task/version.json     # resolved benchmark version
  agent/result.json     # { agent, started_at, ended_at, exit_code }
  agent/version.json    # resolved agent version
  agent/stdout.log
  agent/stderr.log
  model/trajectory.jsonl  # every LLM request+response (LiteLLM format)
  model/result.json     # { model, provider, total_tokens, cost_usd }
```

### Key directories

| Path | Purpose |
|---|---|
| `src/` | Rust CLI (subcommands: build, push, list, images, inspect, prune, run, report) |
| `benchmarks/<name>/` | Dockerfile + compose.yaml per benchmark |
| `agents/<name>/` | Dockerfile per agent |
| `models/<name>/` | Dockerfile + config.yaml per model |
| `core/` | Shared base images (entrypoint, litellm, agent-base-*, benchmark-base-*) |
| `compose/` | Shared compose fragments (evaluate.yaml, services.yaml) |
| `tests/` | Integration test suite organized by category |

### Test organization

Tests are Rust integration tests in `tests/<category>/test.rs`, registered in `Cargo.toml`. Categories:

- `sanity/` — fast file-I/O checks, always run on `cargo test` (no Docker)
- `build/` — container build sweep (`--ignored`)
- `replay/` — recorded-trajectory replay, deterministic, no API keys (`--ignored`)
- `upstream/` — network reachability checks (release only, `--ignored`)
- `live/` — live inference with real API keys (release only, `--ignored`)
- `fleet/` — aggregator that produces `tests/fleet/report.md`

### Env var namespace

All Eval Containers-controlled variables are prefixed `EVAL_`. Key ones:
- Axis selection: `EVAL_BENCHMARK`, `EVAL_AGENT`, `EVAL_MODEL`, `EVAL_TASK_ID`
- Container version (which tag to pull): `EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`
- Internal version (what runs inside): `EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`
- Runtime: `EVAL_TIMEOUT` (seconds, default 300), `EVAL_REGISTRY`, `EVAL_MODEL_MAX_BUDGET` (USD, default $1)

## Rules and conventions

All rules are normative. The full rules graph is rooted at `RULES.md` with per-area documents at `benchmarks/RULES.md`, `agents/RULES.md`, `models/RULES.md`, `compose/RULES.md`, `src/RULES.md`, and `tests/RULES.md`. Read these before modifying anything in their area.

Key rules to internalize:
- Every `EVAL_*` env var must have a matching `--kebab-case` CLI flag.
- All `apt-get install` must be followed by `rm -rf /var/lib/apt/lists/*` **in the same RUN**.
- `pip install` must use `--no-cache-dir`.
- Agent images must pin their upstream version in `ARG <NAME>_VERSION=<semver>` and label it `eval.agent.version`.
- Every benchmark requires `LABEL eval.benchmark.released="true"` before a replay fixture exists — only add the label after a fixture lands.
- The `count_reconciliation` test in `tests/sanity/check.rs` reads the exact number of benchmarks and agents claimed in README.md and diffs against the filesystem. Update the README count when adding a benchmark or agent.

## Per-task benchmarks

`swe-bench`, `compilebench`, and `terminal-bench` use `ARG TASK_ID` at build time — each task is a separate image. Never build them all. Build one task at a time:

```bash
eval-containers build bench swe-bench --task-id sympy__sympy-24066
```

## Apple Silicon / Rosetta

Most benchmarks are `linux/amd64`. Enable Rosetta in Docker Desktop (Settings → General → "Use Rosetta for x86_64/amd64 emulation") — without it, Python-heavy benchmarks will segfault under QEMU.
