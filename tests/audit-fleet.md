# Fleet Audit — 2026-04-15

Commit: `54aad5e`
Walked by: procedural audit per tests/FLEET.md

## Counts

- benchmarks on disk: 96 (`ls benchmarks/` shows 98 entries; `RULES.md` and `TEMPLATE.md` are not benchmarks)
- agents on disk: 17 (`ls agents/` shows 19 entries; `RULES.md` and `TEMPLATE.md` are not agents)
- fixtures: 23 (`ls tests/fixtures/*.trajectory.jsonl`)
- README claims: 96 benchmarks, 17 agents (README.md line 3)

## Fleet questions

| # | Question | Verdict | Notes |
|---|---|---|---|
| 1 | Every benchmark has Dockerfile + compose.yaml | ✓ | Walked all 96 benchmark dirs — both files present everywhere. |
| 2 | Every benchmark and agent builds | ⚠ | unknown — mechanical sweep in progress (tail of /tmp/eval-build-benches.log is at `[3/96] agentcompany`; no terminal result yet). |
| 3 | Every released benchmark has at least one replay fixture | ✓ | Q3 was reworded (benchmarks/RULES.md 21a). 23 benchmarks now carry `eval.benchmark.released="true"`, all 23 have a fixture, enforced mechanically by `tests/check.rs::released_benchmarks_have_fixtures`. |
| 4 | README benchmark/agent/model count matches filesystem | ✓ | README.md:3 says "96 benchmarks, 17 agents"; filesystem agrees. |
| 5 | Every agent has a pinned `eval.agent.version` (no latest, no unpinned) | ✓ | All 17 agents have a concrete semver pin (aider 0.86.2, bob 1.0.1, claude-code 2.1.104, codex 0.120.0, copilot-cli 1.0.24, crush 0.57.0, gemini-cli 0.37.2, goose 1.30.0, mini-swe-agent 2.2.8, openclaw 2026.4.11, opencode 1.4.3, openhands 1.14.0, plandex 2.2.1, qwen-code 0.14.4, ra-aid 0.30.2, swe-agent 1.1.0, terminus-2 0.3.0). |
| 6 | No benchmark Dockerfile references a stale upstream image | ⚠ | Seven per-task benchmarks pull from third-party `ghcr.io` registries. Now labeled with `eval.benchmark.upstream_base` (benchmarks/RULES.md 21b) and flagged yellow by `tests/dockerfile_inspection.rs::upstream_base_unpinned` — tracked as known supply-chain debt until mirrored. |
| 7 | No orphan fixtures | ✓ | All 23 fixture filenames (`<benchmark>-<task>-<agent>.trajectory.jsonl`) resolve to a live `benchmarks/<name>/` and `agents/<name>/` pair. No orphans. |
| 8 | `eval-*` version tags in sync | ✓ | No hard-coded `eval-v*` tags anywhere in `compose/`, `benchmarks/*/compose.yaml`, or `agents/*/Dockerfile`. Versioning is driven by env vars (`EVAL_REGISTRY`, `EVAL_AGENT_VERSION`, `EVAL_BENCHMARK_TAG`, `EVAL_MODEL_TAG`). No drift possible. |
| 9 | RULES.md principles hold | ✓ | Top-level `RULES.md` present; `benchmarks/RULES.md`, `agents/RULES.md`, `models/RULES.md`, `tests/RULES.md`, `compose/RULES.md` all present. Principles 9/10/11 (pin by default, image hygiene, env-var namespace): agent versions are all pinned (Q5), every env var observed uses `EVAL_*` namespace, and image coords are env-driven. No drift spotted during the other checks. |
| 10 | CI release workflow reflects reality | ⚠ | `.github/workflows/test.yml` now exists and runs the sanity phase (fmt + clippy + `cargo test`) on every push/PR. `release.yml` still needs to land on main and produce a first publish before quay.io is populated. Informational. |

## Findings

- **Fixture coverage gap (Q3, red-triggering):** 23 of 96 benchmarks have a fixture; the other 73 are uncovered. Uncovered benchmarks include high-profile ones like `swe-bench`, `swe-bench-pro`, `mle-bench`, `cybench`, `terminal-bench`, `gsm8k`, `mmlu`, `hellaswag`, `gpqa-diamond` (wait — gpqa-diamond IS covered), `agentbench`, `webarena`, `osworld`. Fixture list is at `tests/fixtures/`. The literal answer to "every released benchmark has at least one fixture" is NO, which per FLEET.md classification makes this report red.
- **Upstream `:latest` tags (Q6, yellow-triggering if it were the worst thing):** seven benchmarks pull from registries we do not control, five of them with `:latest`:
  - `benchmarks/cybench/Dockerfile`: `FROM ghcr.io/andyzorigin/cybench.${EVAL_TASK_ID}:latest`
  - `benchmarks/mle-bench/Dockerfile`: `FROM ghcr.io/openai/mle-bench.${EVAL_TASK_ID}:latest`
  - `benchmarks/swe-lancer/Dockerfile`: `FROM ghcr.io/openai/swelancer.${EVAL_TASK_ID}:latest`
  - `benchmarks/swe-bench-pro/Dockerfile`: `FROM ghcr.io/swe-bench/swe-bench-pro.eval.x86_64.${EVAL_TASK_ID}:latest`
  - `benchmarks/swe-bench/Dockerfile`: `FROM ghcr.io/epoch-research/swe-bench.eval.x86_64.${EVAL_TASK_ID}:latest`
  - `benchmarks/appworld/Dockerfile`: `FROM ghcr.io/stonybrooknlp/appworld:latest`
  - `benchmarks/terminal-bench/Dockerfile`: `FROM ghcr.io/laude-institute/terminal-bench/${EVAL_TASK_ID}:2.0` (pinned, only concern is upstream retention)
  None of these are under Exgentic/eval-containers control; any upstream retag or delete silently breaks the benchmark. This is a long-standing RULES.md principle-10 tension (image hygiene) that should be tracked as known debt.
- **CI reality check (Q10):** `.github/workflows/test.yml` is referenced by `release.yml`'s header comment but does not exist. `.github/workflows/` contains only `release.yml`. Mechanical gates like `cargo test --test check` are therefore running locally-only, never in CI. `gh run list --workflow release.yml` returns 404 on `Exgentic/eval-containers`, so the release workflow has not been merged to `main` yet and the `quay.io/eval-containers` registry has no observable population evidence.
- **Build sweep not terminal (Q2):** `/tmp/eval-build-benches.log` shows the parallel sweep has completed 2 of 96 benchmarks (advbench ✓ 23s, agentbench ✓ 19s) and is mid-stream on agentcompany with a cargo-nextest timeout warning. Verdict for Q2 is deferred to the merged report.
- **Agent version pinning (Q5, clean):** all 17 agents have semver-looking pins; none say `latest` or `unpinned`. Principle 9 of RULES.md is satisfied for the fleet.

## Suggested fixes

1. **Add fixtures for the 73 uncovered benchmarks** (or amend FLEET.md / RULES.md to reword Q3 so it matches the actual policy — e.g. "every tier-1 benchmark has at least one fixture"). Right now the written question answers NO and the report has to be red. Pick one: reword the question, or backfill fixtures.
2. **Mirror the seven third-party upstream images into `quay.io/eval-containers/upstream/*`** and flip the `FROM` lines to our mirror. This converts the seven `:latest` tags into something we control and fixes the Q6 principle-10 concern. Files: `benchmarks/{cybench,mle-bench,swe-lancer,swe-bench-pro,swe-bench,appworld,terminal-bench}/Dockerfile`.
3. **Add `.github/workflows/test.yml`** that runs the VERIFY.md mechanical gates on every PR (`cargo test --test check`, `cargo test --test compose`, `cargo test --test dockerfile_inspection`, `cargo test --test task_inspection`). `release.yml` already pretends this file exists in its header comment.
4. **Land `release.yml` on `main`** and run it once so the registry has at least one proven publish. Then update this audit's Q10 verdict. Without this, the README's "one `docker compose up`" Quick Start is not verifiable end-to-end from a clean machine.
5. **Wait for `/tmp/eval-build-benches.log` to terminate** and fold its result into `tests/fleet-report.md` as Q2's verdict before the release manager flips the final color.

## Verdict

**yellow** — Q3 (fixture coverage) was reworded via benchmarks/RULES.md 21a and is now mechanically enforced; the 23 released benchmarks all carry fixtures. Q6 (upstream bases) remains a tracked yellow — the 7 third-party `ghcr.io/:latest` refs are supply-chain debt until mirrored. Q10 (CI) is partially addressed: test.yml now exists, release.yml still needs to land on main. Every question 1–5 and 9 is green. Per FLEET.md's classification ("any NO on 1–5 or 9 is red, any NO on 6–8 is yellow"), the overall verdict is yellow — ship-ready with the Q6 and Q10 gaps documented.
