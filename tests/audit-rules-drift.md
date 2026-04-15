# Rules Drift Audit — 2026-04-15

Commit: `0eb98fc`
Walked by: procedural audit per VERIFY.md step 29

## Per-rule findings

### RULES.md principle 2 — Standalone artifacts

- Verdict: ⚠ partial drift
- Evidence: `compose/evaluate.yaml` (the production artifact) is self-contained and only uses `DOCK_*` env vars plus `env_file: .env`. However, every `benchmarks/<name>/compose.yaml` references `../../.env`, `../../compose/services.yaml`, and `../../output/...` — these are the `--local` dev artifacts per the comment in evaluate.yaml, but nothing in the rule carves out that distinction, and `dock run --local` is a documented production path. A user who pulls a single benchmark directory with `docker compose up` fails on the `env_file: ../../.env` reference.
- Suggested fix: either clarify in RULES.md principle 2 that per-benchmark compose files are the dev workflow and only `compose/evaluate.yaml` is the standalone artifact, or make per-benchmark compose files relative-path-free.

### RULES.md principle 9 — Pin by default, two orthogonal knobs (tag vs. version)

- Verdict: ✗ drifted (systemic)
- Evidence: The rule says container version is selected by image tag via `DOCK_BENCHMARK_TAG` / `DOCK_AGENT_TAG` / `DOCK_MODEL_TAG`, and internal upstream version is selected at runtime via `DOCK_BENCHMARK_VERSION` / `DOCK_AGENT_VERSION` / `DOCK_LITELLM_VERSION` — and the entrypoint MUST read the version env var.
  - 95 of 96 per-benchmark compose files use `${DOCK_AGENT_VERSION:-latest}` as the eval **image tag** (e.g. `benchmarks/aime/compose.yaml:17`). This conflates the two axes: the "internal version" env var is being used as the "container tag". Zero compose files reference `DOCK_AGENT_TAG`.
  - Zero benchmark Dockerfiles reference `DOCK_BENCHMARK_VERSION` anywhere (grep of `benchmarks/*/Dockerfile` returns nothing). The shared `core/entrypoint/dock-entrypoint.sh` does not read it either.
  - Zero agent Dockerfiles or embedded `entrypoint.sh` heredocs reference `DOCK_AGENT_VERSION`. Example: `agents/claude-code/Dockerfile:40-56` — the entrypoint only reads `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, and `TASK`.
  - Zero model images reference `DOCK_LITELLM_VERSION`. `models/claude-opus-4/Dockerfile` is 9 lines and has no entrypoint override.
  - Zero benchmark Dockerfiles declare `ARG DATA_REVISION=` (rule text says "`ARG DATA_REVISION=<sha>` or equivalent"). The revision hash lives in the label and the data-fetch URL, giving no build-time override.
  - `src/run.rs` correctly exposes `--benchmark-tag/--agent-tag/--model-tag` and `--benchmark-version/--agent-version/--litellm-version` as separate flags, but they map to env vars that the images never actually consume.
- Suggested fix: either (a) implement the version-override path in the shared entrypoints and in per-benchmark/per-agent entrypoints, and rewrite the 95 compose files to use `DOCK_AGENT_TAG` as the image tag; or (b) retract the "two orthogonal knobs" split from principle 9 and consolidate to a single version axis. This is the single highest-impact drift in the tree.

### RULES.md principle 10b — In-layer cleanup

- Verdict: ✓ compliant (already enforced)
- Evidence: `tests/dockerfile_inspection.rs` rules `apt_no_cleanup`, `pip_no_cache_flag`, `phantom_pip_uninstall` cover this and run on every `cargo test`.

### RULES.md principle 10a — Slim bases

- Verdict: ⚠ partial drift
- Evidence: `python_full_base` rule in `dockerfile_inspection.rs` catches `FROM python:X` without `-slim`. But the "slim bases" principle also implicitly argues for `--no-install-recommends` on Debian/Ubuntu, which is the single highest-leverage flag for image thinness. 112 of 112 Dockerfiles that use `apt-get install` omit `--no-install-recommends` (none use it). The rule text does not explicitly require it, so this is arguably a missing clarification rather than drift.
- Suggested fix: either add `--no-install-recommends` to principle 10a explicitly and add a mechanical check, or leave as implicit guidance.

### RULES.md principle 11 — `DOCK_*` env var namespace

- Verdict: ⚠ partial drift
- Evidence: The rule says all Dock-controlled env vars MUST be prefixed with `DOCK_`, and "upstream env vars (`OPENAI_API_KEY`, `HF_TOKEN`, etc.) are untouched". Two Dock-controlled vars are unprefixed:
  - `TASK` — mandated by `agents/RULES.md` rule 2 ("The entrypoint MUST read the task from the `TASK` environment variable"). `compose/services.yaml:25` and `compose/evaluate.yaml:38` both pass it through as `TASK`.
  - `EXPECTED_ANSWER` — exported by per-benchmark entrypoints and passed through `compose/services.yaml:27`, `compose/evaluate.yaml:39`.
  Both are Dock-defined (not upstream CLI tools' vars) yet both are unprefixed. The two rules contradict.
- Suggested fix: decide whether these are "upstream-style" agent-facing contract vars (exempt from the DOCK_ namespace, like `OPENAI_API_KEY`) or Dock-controlled vars that should be renamed to `DOCK_TASK` / `DOCK_EXPECTED_ANSWER`. Either way, make principle 11 explicit about the exemption.

### RULES.md principle 12 — Self-contained repository

- Verdict: ✓ compliant
- Evidence: No `.claude/`, `.cursor/`, `.vscode/` load-bearing content in the tree.

### RULES.md principle 13 — Verification is normative

- Verdict: ✓ compliant
- Evidence: `tests/VERIFY.md`, `tests/DOCKERFILE.md`, `tests/TRAJECTORY.md`, `tests/FLEET.md`, `tests/fleet-report.md` all present. `tests/check.rs`, `tests/dockerfile_inspection.rs`, `tests/task_inspection.rs`, `tests/compose.rs`, `tests/fleet.rs` provide the mechanical gates.

### benchmarks/RULES.md rule 1 — Standalone / TASK_ID terminology

- Verdict: ⚠ stale text
- Evidence: Rule 1 says "`TASK_ID` is the only required runtime input" (unprefixed). But `compose/services.yaml:26` and all compose files use `DOCK_TASK_ID`. The axis rename landed in a 2026-04-14 changelog entry on this file, yet rule 1's body text still says `TASK_ID`.
- Suggested fix: s/`TASK_ID`/`DOCK_TASK_ID`/ in rule 1.

### benchmarks/RULES.md rule 3 — Reproducible by default via `ARG DATA_REVISION`

- Verdict: ⚠ partial drift
- Evidence: Rule 3 says the dataset revision MUST be pinned as `ARG DATA_REVISION=<sha>` **or equivalent**. Zero benchmarks use `ARG DATA_REVISION`. 83 of 96 benchmarks pin via a literal hash in `LABEL dock.benchmark.data_revision="..."` which is label-only — not overridable at build time without editing the Dockerfile. The "or equivalent" clause covers the label, but loses the build-arg override path.
- Suggested fix: either tighten rule 3 to require `ARG DATA_REVISION` (and add a mechanical check), or clarify that the label alone satisfies the rule.

### benchmarks/RULES.md rule 4 — Runtime version override reads `DOCK_BENCHMARK_VERSION`

- Verdict: ✗ drifted
- Evidence: Rule text says the entrypoint MUST read `DOCK_BENCHMARK_VERSION`, fetch/materialize that revision, and write `/output/task/version.json` before the agent runs. `core/entrypoint/dock-entrypoint.sh` never reads `DOCK_BENCHMARK_VERSION` and never writes `/output/task/version.json`. No per-benchmark entrypoint reads it either (grep of `benchmarks/*/Dockerfile`).
- Suggested fix: implement the override path in `core/entrypoint/dock-entrypoint.sh` and add a mechanical check that `DOCK_BENCHMARK_VERSION` appears in the shared entrypoint.

### benchmarks/RULES.md rule 21a — Released label gates fixtures

- Verdict: ✓ compliant (already enforced)
- Evidence: `tests/check.rs::released_benchmarks_have_fixtures` enforces this mechanically.

### benchmarks/RULES.md rule 21b — `upstream_base` label

- Verdict: ✓ compliant
- Evidence: All 7 benchmarks with third-party `ghcr.io/*` bases (`cybench`, `mle-bench`, `swe-bench-pro`, `terminal-bench`, `appworld`, `swe-bench`, `swe-lancer`) declare `LABEL dock.benchmark.upstream_base=`. The `upstream_base_unpinned` rule in `tests/dockerfile_inspection.rs` catches `:latest`.

### agents/RULES.md rule 13 — Runtime version override reads `DOCK_AGENT_VERSION`

- Verdict: ✗ drifted
- Evidence: Rule says the entrypoint MUST read `DOCK_AGENT_VERSION`, install/activate that version, and write `/output/agent/version.json`. Example: `agents/claude-code/Dockerfile:40-56` embeds an entrypoint that hardcodes `@anthropic-ai/claude-code@2.1.104` from the install layer and never reads `DOCK_AGENT_VERSION`. No agent writes `/output/agent/version.json`. Same pattern across every agent image.
- Suggested fix: the version-override path does not exist in any agent image. Either implement a common pattern or retract the rule.

### agents/RULES.md rule 14 — `dock.agent.version` label

- Verdict: ✓ compliant (already enforced)
- Evidence: `tests/check.rs::check_agent_structure` requires `LABEL dock.agent.version=` and rejects `"latest"`.

### models/RULES.md rule 13 — `DOCK_LITELLM_VERSION` entrypoint override

- Verdict: ✗ drifted
- Evidence: No model image reads `DOCK_LITELLM_VERSION`. `models/claude-opus-4/Dockerfile` is 9 lines and has no entrypoint. `compose/evaluate.yaml:61` passes the env var through to the container but nothing consumes it. No `/output/model/version.json` is written.
- Suggested fix: same as benchmarks rule 4 / agents rule 13.

### models/RULES.md rule 15 — Required `dock.model.litellm_version` label

- Verdict: ✗ drifted
- Evidence: Zero model Dockerfiles declare `LABEL dock.model.litellm_version=`. `models/claude-opus-4/Dockerfile:1-9` has only `dock.type`, `dock.model.name`, `dock.model.provider`. Same across `claude-sonnet-4`, `gpt-4.1-mini`, `gpt-5`, `gpt-5.4`, `replay`.
- Suggested fix: add a mechanical check on every `models/*/Dockerfile` and add the labels (or rely on the parent `core/litellm:<pin>` base and inherit).

### compose/RULES.md rule 9 — Parameterized by `TASK_ID`, `DOCK_AGENT`, `DOCK_MODEL`, `DOCK_REGISTRY`

- Verdict: ⚠ stale text
- Evidence: Rule says compose files MUST be parameterized by `TASK_ID`. All actual compose files use `DOCK_TASK_ID`. Same rename drift as benchmarks rule 1.
- Suggested fix: s/`TASK_ID`/`DOCK_TASK_ID`/ in rule 9.

### compose/RULES.md rule 16 — Model `result.json` schema

- Verdict: ⚠ unverified / likely drifted
- Evidence: Rule says `/output/model/result.json` MUST contain `model`, `provider`, `total_tokens`, `cost_usd`. `models/replay/server.py` never writes `result.json` (grep returns zero matches in the file). Replay is the primary model image exercised by `tests/replay.rs`, so the schema is unverified — and if the real LiteLLM base image writes it, it's separated from the rule by the base-image boundary.
- Suggested fix: add a replay-test assertion on `output/<benchmark>/<task>/model/result.json` schema.

### tests/RULES.md rule 8 — Replay model writes `trajectory.json` and `result.json`

- Verdict: ⚠ stale text / drifted
- Evidence: Rule 8 says the replay model "MUST write `trajectory.json` and `result.json` to `/output/model/`". On disk: fixtures are `.trajectory.jsonl` (JSON Lines), not `.json`. `models/replay/server.py:22` reads from `/data/trajectory.jsonl`. `benchmarks/RULES.md` rule 21a correctly uses the `.trajectory.jsonl` extension; `tests/RULES.md` rules 8 and 9 still say `.json`. Also, server.py never writes `result.json`.
- Suggested fix: update rules 8 and 9 to `trajectory.jsonl`; implement `result.json` writing in `models/replay/server.py`.

### tests/RULES.md rule 9 — Fixture filename convention

- Verdict: ⚠ stale text
- Evidence: Rule 9 says fixture is `tests/fixtures/{benchmark}-{task-id}-{agent}.trajectory.json`. On disk all fixtures are `.trajectory.jsonl`.
- Suggested fix: s/`.trajectory.json`/`.trajectory.jsonl`/ in rule 9.

## Summary

- ✓ compliant: 7
- ⚠ partial drift: 8
- ✗ drifted: 5

## Top 5 drift findings

- **`compose/evaluate.yaml` + 95 per-benchmark compose files** conflate the two version axes: they use `${DOCK_AGENT_VERSION:-latest}` as the **image tag** when RULES.md principle 9 says tags come from `DOCK_AGENT_TAG` and versions are read by the container entrypoint. Zero compose files reference `DOCK_AGENT_TAG`. The split that principle 9 created in the 2026-04-14 changelog never made it into the compose layer.
- **`core/entrypoint/dock-entrypoint.sh`** (shared benchmark entrypoint) never reads `DOCK_BENCHMARK_VERSION` and never writes `/output/task/version.json`, violating benchmarks/RULES.md rule 4. No per-benchmark entrypoint picks up the slack either.
- **Every agent `entrypoint.sh`** (18 agents) ignores `DOCK_AGENT_VERSION` and never writes `/output/agent/version.json`, violating agents/RULES.md rule 13. Example: `agents/claude-code/Dockerfile:40-56` hardcodes `@anthropic-ai/claude-code@2.1.104`.
- **Every model image** (6 models) is missing the required `LABEL dock.model.litellm_version`, violating models/RULES.md rule 15, and no model image reads `DOCK_LITELLM_VERSION` per rule 13.
- **Rule-text rename lag**: benchmarks/RULES.md rule 1, compose/RULES.md rule 9, tests/RULES.md rules 8 and 9 still say unprefixed `TASK_ID` and `.trajectory.json`. The code renamed to `DOCK_TASK_ID` and `.trajectory.jsonl` and the rule docs were not touched.

## Proposed mechanical checks

- new rule `entrypoint_reads_benchmark_version`: assert `core/entrypoint/dock-entrypoint.sh` contains a reference to `DOCK_BENCHMARK_VERSION` and writes `/output/task/version.json` (benchmarks/RULES.md rule 4).
- new rule `agent_entrypoint_reads_agent_version`: assert every `agents/*/Dockerfile` embeds an `entrypoint.sh` heredoc that references `DOCK_AGENT_VERSION` (agents/RULES.md rule 13).
- new rule `model_has_litellm_version_label`: assert every `models/*/Dockerfile` contains `LABEL dock.model.litellm_version=` (models/RULES.md rule 15).
- new rule `model_entrypoint_reads_litellm_version`: assert model images reference `DOCK_LITELLM_VERSION` (models/RULES.md rule 13).
- new rule `compose_uses_agent_tag_for_image`: assert no benchmark `compose.yaml` uses `DOCK_AGENT_VERSION` as a Docker image tag placeholder — only `DOCK_AGENT_TAG` is permitted in the `image:` field (RULES.md principle 9).
- new rule `apt_no_install_recommends`: yellow-level check that `apt-get install` is followed by `--no-install-recommends` (RULES.md principle 10a clarification).
- new rule `arg_data_revision`: red-level check that every benchmark Dockerfile that fetches a dataset declares `ARG DATA_REVISION=` (benchmarks/RULES.md rule 3 tightened).
- new rule `fixture_extension_jsonl`: red-level check that every file under `tests/fixtures/` ends in `.trajectory.jsonl` (aligns with benchmarks/RULES.md rule 21a; would catch any stragglers).
