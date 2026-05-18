# How to verify Eval Containers

Eval Containers is verified on **two axes**:

- **Mechanical** — deterministic checks, run by `cargo test` or an external
  tool, produce pass/fail.
- **Procedural** — judgment-level review, walked by a human or a sub-agent
  following a markdown checklist, produces a written verdict.

Both axes exist at three scales: per-file, per-image, whole-fleet. Mechanical
catches what's broken. Procedural catches what's wrong but passing the rules.
Neither replaces the other.

This document is the **complete release checklist**: every step, every
executor, every artifact. Nothing ships without walking it.

## The complete process

| #  | Phase       | Step                                                      | Kind        | How                                                              | Artifact / pass criterion                               |
|----|-------------|-----------------------------------------------------------|-------------|------------------------------------------------------------------|---------------------------------------------------------|
| 1  | Preflight   | Clean working tree, on release branch                    | manual      | `git status`                                                     | no untracked / modified files                           |
| 2  | Preflight   | Read last release report                                 | manual      | open `tests/fleet-report.md`                                     | know what was flagged last time                         |
| 3  | Preflight   | Read release notes draft                                 | manual      | open `RELEASE.md` / `CHANGELOG.md`                               | know what's shipping                                    |
| 4  | Sanity      | Rust formatting + lint                                   | mechanical  | `cargo fmt --check && cargo clippy -- -D warnings`               | zero warnings                                           |
| 5  | Sanity      | Rule-engine unit tests (internal consistency)            | mechanical  | `cargo test`                                                     | all rule tests green                                    |
| 6  | Sanity      | Structural validation (files present, labels present)   | mechanical  | `cargo test --test check structure`                              | 96 benchmarks + 17 agents pass                          |
| 7  | Sanity      | Compose config parse                                     | mechanical  | `cargo test --test check compose`                                | 96/96 parse via `docker compose config`                 |
| 8  | Sanity      | Dockerfile health inspection (12-rule catalog)           | mechanical  | `cargo test --test check dockerfile`                             | 113/113 green, 0 red                                    |
| 9  | Sanity      | Trajectory health inspection (13-rule catalog)           | mechanical  | `cargo test --test check trajectory`                             | 23/23 fixtures green                                    |
| 10 | Sanity      | Count reconciliation (README claims vs filesystem)       | mechanical  | `cargo test --test check counts`                                 | benchmarks / agents / models match README               |
| 11 | Build       | Build every core image                                   | mechanical  | `cargo test --test build core -- --ignored`                      | 4/4 (entrypoint, test-exact-match, litellm, llm-bridge) |
| 12 | Build       | Build every benchmark image                              | mechanical  | `cargo test --test build benchmarks -- --ignored`                | 96/96, or known-failing list diffed vs prior run        |
| 13 | Build       | Build every agent image                                  | mechanical  | `cargo test --test build agents -- --ignored`                    | 17/17                                                   |
| 14 | Build       | Build every model image                                  | mechanical  | `cargo test --test build models -- --ignored`                    | N/N                                                     |
| 15 | Replay      | Replay every recorded fixture                            | mechanical  | `cargo test --test replay -- --ignored`                          | 23/23 trajectories reproduce same score                 |
| 16 | End-to-end  | One fresh live run (smoke)                               | mechanical  | `cargo test --test run smoke -- --ignored`                       | score file exists, score ∈ [0, 1]                       |
| 17 | End-to-end  | Eyeball the smoke trajectory                             | manual      | read `/tmp/eval-smoke/trajectory.jsonl`                          | agent saw a real task, response looks sane              |
| 18 | Upstream    | Every pinned dataset revision still resolves            | mechanical  | `cargo test --test upstream datasets -- --ignored`               | no 404s on HuggingFace / GitHub raw URLs                |
| 19 | Upstream    | Every pinned pip / npm version still exists              | mechanical  | `cargo test --test upstream packages -- --ignored`               | no yanked or removed versions                           |
| 20 | Upstream    | Every `FROM` base image still pullable                  | mechanical  | `cargo test --test upstream bases -- --ignored`                  | no dangling base refs                                   |
| 21 | Upstream    | hadolint scan (optional external linter)                | mechanical  | `hadolint $(find . -name Dockerfile)`                            | zero errors, warnings reviewed                          |
| 22 | Security    | Secret scan (gitleaks)                                   | mechanical  | `gitleaks detect --source . --no-git`                            | zero findings                                           |
| 23 | Audit       | Walk [DOCKERFILE.md] — new or changed Dockerfiles        | procedural  | human or sub-agent, 7 questions per file                         | yes / no / n.a. with one-line reason per question       |
| 24 | Audit       | Walk [TRAJECTORY.md] — new or changed fixtures           | procedural  | human or sub-agent, 5 questions per fixture                      | written verdict per fixture                             |
| 25 | Audit       | Walk [FLEET.md] — the 10 release questions               | procedural  | human or sub-agent, whole repo                                   | 10 answers, red / yellow / green verdict                |
| 26 | Audit       | Agent roster still representative of the field          | manual      | eyeball `agents/` vs current state of the art                    | note any gaps                                           |
| 27 | Audit       | Benchmark set still representative                      | manual      | eyeball `benchmarks/` vs recent arxiv + leaderboards             | note any gaps                                           |
| 28 | Docs        | README Quick Start works from clean clone               | manual      | fresh clone, follow README verbatim                              | runs end-to-end without edits                           |
| 29 | Docs        | RULES.md still matches actual repo state                | manual      | read RULES.md against recent commits                             | no drift                                                |
| 30 | Docs        | Every benchmark has a `README.md`                       | mechanical  | `cargo test --test check benchmark_readmes`                      | 96/96                                                   |
| 31 | Docs        | Every agent has a `README.md`                           | mechanical  | `cargo test --test check agent_readmes`                          | 17/17                                                   |
| 32 | Docs        | CHANGELOG entry written                                  | manual      | edit `CHANGELOG.md`                                              | one bullet per user-visible change                      |
| 33 | CI          | CI is green on `main`                                    | manual      | GitHub Actions / `gh pr checks`                                  | all workflows green                                     |
| 34 | CI          | Release workflow has run recently                       | manual      | check last run timestamp                                         | ≤ 7 days old                                            |
| 35 | Fleet       | Generate the mechanical half of the report              | mechanical  | `cargo test --test fleet -- --ignored`                           | `tests/fleet-report.md` (auto section)                  |
| 36 | Fleet       | Paste audit answers into the report                     | manual      | edit `tests/fleet-report.md`                                     | manual section filled in                                |
| 37 | Fleet       | Classify the overall verdict                            | manual      | apply FLEET.md rules to the report                               | red / yellow / green                                    |
| 38 | Release     | Tag the commit                                           | manual      | `git tag -s eval-vX.Y.Z`                                         | signed tag                                              |
| 39 | Release     | Push tag + trigger release workflow                     | manual      | `git push origin eval-vX.Y.Z`                                    | workflow running                                        |
| 40 | Release     | Verify images published to `quay.io`                    | manual      | `docker pull quay.io/eval-containers/<image>:eval-vX.Y.Z` on each     | every expected tag exists                               |
| 41 | Release     | Verify image signatures / attestations                  | manual      | `cosign verify quay.io/eval-containers/<image>:eval-vX.Y.Z`            | signatures valid                                        |
| 42 | Release     | Smoke test one image from a clean machine               | manual      | pull + `eval-containers run` from a different host                          | end-to-end works from nothing                           |
| 43 | Release     | Attach `fleet-report.md` to GitHub release              | manual      | `gh release create ... --notes-file tests/fleet-report.md`       | release visible                                         |
| 44 | Post        | Announce release (Slack / Discord / site)               | manual      | paste release notes                                              | linked release URL                                      |
| 45 | Post        | File follow-up issues for yellow findings               | manual      | `gh issue create` per yellow                                     | every known gap has an issue                            |
| 46 | Post        | Archive this run's `fleet-report.md` with the tag       | manual      | stored alongside the tag                                         | can diff against next release                           |

## Grouped by executor

| Executor                    | Steps                                                                 | Count |
|-----------------------------|-----------------------------------------------------------------------|-------|
| **`cargo test`** (machine)  | 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 30, 31, 35 | 19    |
| **External tools**          | 21 (hadolint), 22 (gitleaks)                                         | 2     |
| **Human or sub-agent audit**| 23, 24, 25                                                           | 3     |
| **Human only**              | 1, 2, 3, 17, 26, 27, 28, 29, 32, 33, 34, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46 | 22 |

## Grouped by frequency

| Frequency        | Steps                                           |
|------------------|-------------------------------------------------|
| Every commit     | 4, 5, 6, 7, 8, 9, 10                            |
| Every PR         | above + 11, 12, 13, 14                          |
| Every release    | **all 46**                                      |
| Quarterly drift  | 18, 19, 20, 23, 24, 25, 26, 27, 29              |

## The two axes, at a glance

|              | Mechanical (machine)                    | Procedural (human or sub-agent)                |
|--------------|------------------------------------------|------------------------------------------------|
| per-file     | step 8 (Dockerfile), step 9 (trajectory) | step 23 ([DOCKERFILE.md]), step 24 ([TRAJECTORY.md]) |
| per-image    | steps 11–14 (build sweep)                | —                                              |
| end-to-end   | step 16 (live smoke)                     | step 17 (eyeball trajectory)                   |
| whole-fleet  | step 35 (fleet report, auto section)     | step 25 ([FLEET.md])                           |

## The release flow

```
1.  git status clean, read last report, read release notes              (steps 1–3)
2.  cargo test                                                          (steps 4–10, fast)
3.  cargo test --test build -- --ignored                                (steps 11–14, slow)
4.  cargo test --test replay -- --ignored                               (step 15)
5.  cargo test --test run -- --ignored                                  (steps 16–17)
6.  cargo test --test upstream -- --ignored                             (steps 18–20)
7.  hadolint + gitleaks                                                 (steps 21–22)
8.  walk DOCKERFILE.md / TRAJECTORY.md / FLEET.md                       (steps 23–27)
9.  README quick-start, RULES.md sanity, CHANGELOG                      (steps 28–32)
10. CI green, release workflow recent                                   (steps 33–34)
11. cargo test --test fleet -- --ignored                                (step 35)
12. paste audit answers, classify verdict                               (steps 36–37)
13. tag, push, publish, verify, announce, follow up                     (steps 38–46)
```

A release is **not ready** until every row on the 46-row table has been
executed and has its artifact or pass mark recorded in `fleet-report.md`.

## Audit procedure (steps 23–25, 26–27)

The procedural checklists are **toolchain-agnostic by design**. They never
say "ask Claude" or "use GPT". They describe the questions in plain
language; the reader answers yes / no / n.a. with a one-line reason. A human
can walk them in their editor. A sub-agent can walk them in batch — dispatch
one sub-agent per Dockerfile with the checklist as its task, collect the
verdicts. Both paths produce the same report shape so findings diff cleanly
across releases.

| Checklist           | Scope            | Questions | Typical executor             |
|---------------------|------------------|-----------|------------------------------|
| [DOCKERFILE.md]     | one Dockerfile   | 7         | sub-agent in batch (113 files) |
| [TRAJECTORY.md]     | one fixture      | 5         | sub-agent in batch (23 fixtures) |
| [FLEET.md]          | whole repository | 10        | human (the release manager)  |

## File map

| File                             | Role                                                           |
|----------------------------------|----------------------------------------------------------------|
| `tests/check.rs`                 | fast mechanical: structure, compose, Dockerfiles, fixtures, counts, READMEs |
| `tests/build.rs`                 | per-image build sweep (benchmarks, agents, models, core)        |
| `tests/replay.rs`                | replay recorded trajectory fixtures                             |
| `tests/run.rs`                   | one live end-to-end smoke                                       |
| `tests/upstream.rs`              | upstream verification (datasets, packages, base images)         |
| `tests/fleet.rs`                 | runs the mechanical chain, emits `fleet-report.md`              |
| `tests/DOCKERFILE.md`            | per-Dockerfile procedural checklist (7 questions)              |
| `tests/TRAJECTORY.md`            | per-fixture procedural checklist (5 questions)                 |
| `tests/FLEET.md`                 | whole-fleet procedural checklist (10 questions)                |
| `tests/fleet-report.md`          | the generated release artifact (auto + manual sections)        |

Every Rust test file carries its rule catalog inline as a `const RULES: &[Rule]`
array — one row per rule, with the rule ID matching the entry in the
procedural markdown. The two can't drift.

## Streaming output

Every slow test prints one line per item as it runs:

```
[12/96] swe-bench ✓ 47s
[13/96] webarena ✗ 23s  →  re-run: cargo test --test build -- --ignored webarena
```

Failures include the exact re-run command.

## Status today

Steps 5, 6, 7, 8, 9, 15 exist and pass. Step 13 (agent build) is proven.
Step 12 (benchmark build) is the current long-running sweep. Steps 4, 10,
11, 14, 16–22, 30, 31, 35–46 are not yet implemented.

[DOCKERFILE.md]: DOCKERFILE.md
[TRAJECTORY.md]: TRAJECTORY.md
[FLEET.md]: FLEET.md
