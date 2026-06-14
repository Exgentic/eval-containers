# Changelog

All notable changes to Eval Containers are recorded here. Each release entry lists
what shipped and why, in the voice of the change — not the PR that
landed it.

The format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project roughly follows [Semantic Versioning](https://semver.org/)
applied to the image fleet: the major version is bumped on breaking
changes to the rule catalogs; the minor on a benchmark or agent
addition; the patch on a bug fix that doesn't change the rule surface.

## [Unreleased]

### Added

- **The `eval-containers` CLI is now installable as a published artifact.**
  Apache-2.0 licensed, with crates.io metadata (`cargo install eval-containers`)
  and a [`dist`](https://opensource.axo.dev/cargo-dist/)-driven `release.yml`
  that, on every `v*` tag, builds prebuilt binaries for macOS (Apple Silicon +
  Intel), Linux (x86_64 + aarch64), and Windows — installable with no Rust
  toolchain via the generated `curl … | sh` / PowerShell installers. An
  `include` allowlist keeps the crate tarball to just `src/` + manifests so the
  surrounding 100-benchmark monorepo is not published. The prior image-fleet
  release workflow is renamed to `release-images.yml`.
- **100 benchmarks × 20 agents** in the fleet (up from 96 × 17).
  - New IBM benchmarks: `acpbench` (1040 MCQ), `assetopsbench`
    (152 industrial-asset scenarios), `vakra` (28 multi-hop tool-calling),
    `itbench` (10 CISO scenarios, skeleton).
  - New agents: `cline` (plan/act + MCP), `continue-cli`
    (`cn`, multi-model CLI), `open-interpreter` (NL code exec).
- **Rule 11: "Reuse over repetition"** in [`RULES.md`](RULES.md).
  Any infrastructure concern shared by more than two images MUST be
  factored into a shared base image or helper. Consequences:
  `core/agent-base-{node,python,rust}` and
  `core/benchmark-base-{hf,github,external}` land as canonical bases;
  every agent and most benchmarks extend them.
- **`core/entrypoint/eval-sitecustomize.py`** — single-home urllib retry
  helper. Every benchmark's `RUN python3 <<'PYEOF' urllib.request.urlretrieve(...)`
  silently retries on transient HF / network failures with zero
  per-benchmark changes.
- **`EVAL_BUILD_PARALLEL`** env var on `cargo test --test build`.
  Tokio `JoinSet + Semaphore` parallelise the build sweep; label-check
  phase stays serial for deterministic logging. Drains on panic so no
  `ImageGuard` leaks. Documented in [`tests/LOCAL.md`](tests/LOCAL.md)
  Level 2b.
- **`.gitleaks.toml`** — scoped allowlist for `user_api_key_hash` and
  `prompt_cache_key` inside `tests/fixtures/*.trajectory.jsonl`
  (observability IDs, not credentials).
- **`.agents/delivery/release/references/readiness.md`** — per-release verdict document.

### Changed

- **The k8s `job` mode is now a self-contained Helm chart.** A benchmark
  is selected with `--set benchmark=<x>` instead of
  `-f benchmarks/<x>/values.yaml`; the 4 benchmarks with bespoke topology
  (`osworld`, `tau-bench`, `visualwebarena`, `webarena`) moved into the
  chart as `benchmarks/_chart/presets/<x>.yaml` (loaded via `.Files.Get`),
  and the 98 one-line `values.yaml` files were deleted. The chart now
  renders with no external file, so it can be packaged and published to an
  OCI registry. Renders byte-identical to the prior `-f values.yaml` form.
- **Agent Dockerfiles: 1957 → 585 lines (70% reduction)** across all
  20 agents via the Rule 11 refactor onto shared bases.
- **91 of 100 benchmarks** refactored to extend `core/benchmark-base-*`.
  The 9 that don't (`swe-bench`, `swe-bench-pro`, `swe-lancer`,
  `mle-bench`, `cybench`, `terminal-bench`, `compilebench`, `appworld`,
  `aider-polyglot`) legitimately can't share a base — per-task upstream
  images or multi-language toolchains.
- **`agent-base-python`** switched from `pip` to `uv==0.5.14`. A
  `/usr/local/bin/pip` shim forwards every subclass `pip install` to
  `uv pip` with a 5-shot retry loop (`UV_HTTP_TIMEOUT=120`).
- **`agent-base-node`** sets `npm config set fetch-retries=10
  fetch-retry-maxtimeout=120000` globally; all npm-agent subclasses
  inherit robustness against registry flake.
- **All 8 core images pinned `FROM --platform=linux/amd64`**. Previously,
  agent bases built natively arm64 on Apple Silicon while benchmark
  bases pulled amd64 by default, producing combo images with mixed-arch
  binaries that failed with "cannot execute: required file not found"
  under Rosetta.
- **Model healthcheck in `compose/{evaluate,services}.yaml`** switched
  `/health` → `/health/liveness`. Stock `/health` exercises every
  configured model alias with a real upstream call (~20 round trips);
  under a 5s compose timeout it never reports healthy. `/health/liveness`
  is an instant liveness probe.
- **`tests/build/test.rs` bootstrap** uses `docker build` CLI for core
  images (not testcontainers `build_image`). BuildKit's image-cache
  vs daemon's classic image-store race intermittently broke
  `COPY --from=<just-built-tag>` inside bootstrap chains. Rule 6b
  carve-out per [`tests/containers/RULES.md`](tests/containers/RULES.md)
  rule 1.
- **`tests/upstream/test.rs`** gained `is_first_party()` filter for
  `quay.io/eval-containers/*` self-references — they're locally built, not
  yet published, so probing `docker manifest inspect` on them always
  404s and crowds out real drift signal.

### Fixed

- **Silent apt-retry wrapper**: `A && B && break || retry` swallowed
  the final failure after exhausting retries. All 5 bases now use an
  explicit `ok=0/1` gate that fails the build on unrecoverable apt.
- **`goose` agent's `curl | tar xj` pipe** — no integrity check; partial
  bytes under network flake produced a corrupted bz2. Replaced with
  download-then-verify-size-then-extract, 5-shot retry.
- **`mle-bench` `pip install --target /tests/deps mlebench`** lacked
  a version pin. Pinned to `mlebench==1.4.0`; `cargo test` Dockerfile
  rule catalog now green.
- **Count drift**: `README.md` claimed "96 benchmarks, 17 agents" while
  the filesystem had 100 / 20. Corrected; `count_reconciliation` test
  green.
- **7 missing README files** — new benchmarks (`acpbench`, `assetopsbench`,
  `vakra`, `itbench`) and new agents (`cline`, `continue-cli`,
  `open-interpreter`) all carry their per-directory README now.
- **Stale text in `tests/build/known-broken.md`** ("81/96 pass") replaced
  with the 100-benchmark baseline plus a note on local podman
  concurrent-network saturation.

### Security

- `.env` no longer exposed via compose `env_file:` on the eval service
  (agent container). API keys remain only where the LiteLLM proxy
  needs them (the `model` service). Dummy `ANTHROPIC_API_KEY=sk-proxy`
  and `OPENAI_API_KEY=sk-proxy` populated in `services.yaml` for SDK
  initialization.

### Test infrastructure

- `cargo fmt --check` — green.
- `cargo clippy --all-targets -- -D warnings` — green.
- `cargo test` — green (12 rule-catalog + 19 trajectory + 4 upstream +
  6 sanity = 41 mechanical tests).
- `cargo test --test check` — green (structural / compose / Dockerfile /
  trajectory / counts / README).
- `cargo test --test upstream -- --ignored` — green (first-party filter).
- `cargo test --test build build_every_agent -- --ignored
  --test-threads=1 EVAL_BUILD_FILTER=claude-code,aider` — **2/2 green**.
- `cargo test --test fleet -- --ignored` — yellow (mechanical all
  green, procedural yellow from upstream-base tag-pinning debt).
- `hadolint` — 0 errors, 25 review warnings (117 heredoc false-positives
  from hadolint 2.14 parser limitation).
- `gitleaks detect --source .` — 0 findings after config.
- **Live smoke end-to-end validated**: eval-combo build → model
  container healthcheck → agent exec → LiteLLM trajectory → grader →
  `result.json`. LLM backend itself (`litellm.internal.invalid`)
  is VPC-internal and unreachable without IBM network access — code
  path 100% validated, runtime blocker is environmental.

### CI-side follow-ups (not blocked on code)

- Full 100-benchmark + 20-agent build sweep (`cargo test --test build --
  --ignored --test-threads=1 EVAL_BUILD_PARALLEL=4`) — expected clean
  on Linux Docker in CI; podman-on-macOS saturates the VM network
  under high concurrency.
- Live fleet sweep — requires reachable LLM backend (IBM VPN, or
  alternative provider). Existing `tests/live/checkpoint.json` carries
  471 prior-run entries for resume.
- Replay re-run against fresh eval-combo images built under the new
  bases — deferred to CI.

### Known gaps (yellow, documented, non-blocking)

- `hadolint` 2.14 heredoc parser can't handle `RUN python3 <<'PYEOF'`
  — 117 false-positive parse errors. Upstream issue.
- 3 benchmark bases `COPY --from=quay.io/eval-containers/core/entrypoint:latest`
  with mutable `:latest` tag. Acceptable while the core/* images are
  pre-registry; tighten to a digest once published.
- `core/agent-base-rust` label is `eval.base.runtime="rust"` but also
  hosts Go-based agents (`crush`). Documented in its Dockerfile
  header comment; split into `core/agent-base-go/` if a Go-only base
  becomes worthwhile.
