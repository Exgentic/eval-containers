# Fleet Health Inspection

**Status:** Draft
**Date:** April 2026

## Abstract

Per-file inspections (trajectory health, Dockerfile health) tell you
each benchmark and agent is individually sound. They don't tell you
the **fleet** is sound.

A fleet is sound when:
- Every claim in documentation matches what's on disk.
- Every benchmark has the tests it's supposed to have.
- Every agent and model version referenced from the README actually
  builds.
- No benchmark has silently drifted below the quality bar without
  anyone noticing.
- Nothing is stale — agents haven't been bumped in months, dataset
  pins are still resolvable upstream, CI status reflects reality.

Fleet health inspection is the high-level sanity check you run before
a release, before a pitch, or quarterly as a health audit. It
identifies gaps, regressions, and drift across the whole repo rather
than inside a single file.

## The inspection unit

The whole repository at a given commit. Input is the working tree;
output is a fleet health report.

## Signal catalog

### Red signals (any one: the fleet is not release-ready)

- **Build gap.** A committed benchmark or agent that has never been
  successfully built. `cargo test --test build -- --ignored` fails on
  it, or it has never been run.
- **Missing fixture for a released benchmark.** A benchmark whose
  `eval.benchmark.*` labels declare it ready, but no replay fixture
  exists under `tests/fixtures/`. Cannot be end-to-end verified.
- **Documentation drift.** README claims "96 benchmarks, 17 agents"
  but the filesystem has a different count.
- **Convention drift.** A benchmark uses a pattern that `RULES.md`
  forbids (e.g. unprefixed env vars after the April 2026 migration,
  or `extends:` pointing to a non-existent file after a refactor).
- **CI lies.** The release workflow hasn't run in the last N days but
  the README claims the registry is live. Or vice versa: CI is green
  but the registry is empty.
- **Unresolvable pin.** A benchmark's `eval.benchmark.data_revision`
  points at a sha that returns 404 upstream.

### Yellow signals (worth attention, not blocking)

- **Stale agent.** An agent whose upstream version in
  `eval.agent.version` is more than 90 days behind the project's latest
  release on GitHub / npm / PyPI.
- **Missing per-benchmark README.** A benchmark directory with no
  `README.md` explaining its origin, its dataset, and any quirks.
- **Unreleased benchmark.** A directory committed but not yet pushed
  to the registry. Might be intentional (in-progress) or forgotten.
- **Orphan fixture.** A fixture under `tests/fixtures/` that references
  a benchmark-agent combination no longer in the repo.
- **Model coverage gap.** A benchmark that's never been fixture-tested
  against a specific model axis.
- **Replay coverage gap.** A benchmark with fixtures for one agent
  but none of the other supported agents, so regressions in those
  combinations are silent.

### Green signals (all must be present for a `green` fleet verdict)

- **Every benchmark builds.** Full `cargo test --test build -- --ignored`
  sweep passes.
- **Every Dockerfile is healthy.** `cargo test --test dockerfile_inspection
  -- --ignored` passes with zero red findings.
- **Every trajectory is healthy.** `cargo test --test task_inspection
  -- --ignored` passes on every existing fixture.
- **Counts match docs.** README benchmark/agent/model counts match
  the filesystem.
- **All released benchmarks have fixtures.** Every benchmark marked
  released has at least one replay fixture.
- **All CI workflows green on `main`.**

## Classification rules

```
if any red signal:     fleet verdict = red     # do not ship
elif any yellow:       fleet verdict = yellow  # ship with known gaps documented
else:                  fleet verdict = green   # clean release
```

## Layered checking

**Layer 1 — mechanical counts and cross-references.** Scripted
checks: walk `benchmarks/` and `agents/`, count, compare against
README, diff against `tests/fixtures/` contents, compare label values
with directory names, walk the Dockerfile `FROM` graph and check
every referenced image exists. This layer can be a shell script or a
Rust test; it's deterministic and cheap.

**Layer 2 — procedural audit.** The checklist below. A reviewer walks
the fleet, asks the ten questions, writes a report. Covers the
subjective drift questions: "are these benchmarks still what we claim
they are?", "is the agent roster still representative of the field?",
"have any of the datasets silently changed upstream?"

**Layer 3 — upstream verification (network required).** For every
pinned dataset revision, fetch the manifest from the upstream registry
and verify it still resolves. For every agent's upstream package,
query the package registry for the latest stable version and compare
against the pinned version. Not a CI gate — it needs network and is
slow — but a valuable pre-release check.

## Audit procedure

Run this at least once before each release, and quarterly as a
health check.

### Scope

- **Input:** the whole working tree at a given commit.
- **Context:** the release notes (what are we shipping?), the previous
  audit report (what was flagged last time?), RULES.md (what's
  normative right now?).

### Steps

1. **Run all mechanical gates.** In order:
   ```
   cargo test --test check structural_validation
   cargo test --test compose -- --ignored
   cargo test --test dockerfile_inspection -- --ignored
   cargo test --test task_inspection -- --ignored
   cargo test --test build -- --ignored       # the slow one
   cargo test --test replay -- --ignored      # end-to-end
   ```
   Record pass / fail for each. Anything failing goes in the report.

2. **Count reconciliation.**
   - `ls benchmarks/ | wc -l` vs README claim
   - `ls agents/ | wc -l` vs README claim
   - `ls tests/fixtures/*.trajectory.jsonl | wc -l` vs "how many
     benchmarks have fixtures"

3. **Walk the ten fleet questions.** For each, answer yes / no /
   n.a. with a one-line reason.

   | # | Question |
   |---|---|
   | 1 | Does every benchmark in `benchmarks/` have both a `Dockerfile` and a `compose.yaml`? |
   | 2 | Does every committed benchmark and agent actually build? |
   | 3 | Does every benchmark labeled `eval.benchmark.released="true"` have at least one replay fixture under `tests/fixtures/`? (see benchmarks/RULES.md 21a) |
   | 4 | Does the README's benchmark/agent/model count match the filesystem? |
   | 5 | Does every agent in `agents/` have a pinned `eval.agent.version` label — no `unpinned`, no `latest`? |
   | 6 | Are there any benchmarks whose Dockerfiles reference upstream images we no longer control or that have gone stale? |
   | 7 | Are there any orphan fixtures (fixture files referencing benchmarks / agents no longer in the repo)? |
   | 8 | Is every published `eval-*` version tag in sync — no benchmark at eval-v2 while shared compose is still eval-v1? |
   | 9 | Do the RULES.md principles (esp. 9, 10, 11) hold across every benchmark and agent, or has anything drifted? |
   | 10 | Does the CI release workflow reflect reality — has it run recently, is the registry actually populated, does the README Quick Start work from scratch on a clean machine? |

4. **Classify.** Apply the rules above: any no on 1–5 or 9 is red,
   any no on 6–8 is yellow, question 10 is informational but
   mandatory before shipping.

### Output format

A single markdown report, structured:

```
# Fleet Health Report — YYYY-MM-DD

## Counts
- benchmarks: 96 on disk, 96 claimed in README ✓
- agents: 17 on disk, 17 claimed in README ✓
- fixtures: 23 ✓

## Mechanical gates
- validate-all.sh: ✓
- compose: ✓ (96/96)
- dockerfile_inspection: ✓
- task_inspection: ✓ (23/23 green, 0 yellow)
- build: ⚠ (4 known failures: swe-bench-pro, cybench, mle-bench, swe-lancer — upstream TODO)
- replay: ✓ (23/23)

## Fleet questions
| # | Q                          | Verdict | Notes |
|---|----------------------------|---------|-------|
| 1 | Every bench has both files | ✓       |       |
| 2 | Every bench builds         | ⚠       | 4 TODO |
| ...                                                  |

## Suggested fixes
1. Fix the 4 per-task benchmarks' upstream image references
2. Add fixtures for the 73 benchmarks currently uncovered
3. ...

## Verdict
⚠ yellow — ship-ready with 4 known build gaps documented
```

### When to run

- Before cutting a release (mandatory)
- Quarterly health check (recommended)
- After a batch of ≥10 new benchmarks / agents lands
- After a RULES.md change that touches multiple files

### Who runs it

Anyone. The mechanical gates are `cargo test` — anyone with the repo
clone can run them. The fleet questions are a checklist any reviewer
can walk, whether human, AI assistant, or script. The output format is
fixed so reports can be diffed across time.

## References

- [RULES.md](../RULES.md) — normative principles for the whole repo
- [TRAJECTORY.md](TRAJECTORY.md) — per-fixture runtime health spec
- [DOCKERFILE.md](DOCKERFILE.md) — per-Dockerfile static health spec
- [LOCAL.md](LOCAL.md) — how to run the mechanical gates locally
- [../RELEASE.md](../RELEASE.md) — how CI builds and publishes the fleet
