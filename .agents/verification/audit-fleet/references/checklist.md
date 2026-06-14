# Fleet Health — signal catalog and reference

Reference material for the `audit-fleet` skill. This is the detailed
whole-repository signal catalog, classification rules, and layered-checking
model that back the ten-question walk in `SKILL.md`. Read it before walking the
questions.

## The inspection unit

The whole repository at a given commit. Input is the working tree; output is a
fleet health report.

## Signal catalog

### Red signals (any one: the fleet is not release-ready)

- **Build gap.** A committed benchmark or agent that has never been successfully
  built. `cargo test --test build -- --ignored` fails on it, or it has never
  been run.
- **Missing fixture for a released benchmark.** A benchmark whose
  `eval.benchmark.*` labels declare it ready, but no replay fixture exists under
  `tests/replay/fixtures/`. Cannot be end-to-end verified.
- **Documentation drift.** README claims a count (e.g. "96 benchmarks, 17
  agents") but the filesystem has a different count.
- **Convention drift.** A benchmark uses a pattern that RULES forbids (e.g.
  unprefixed env vars after the April 2026 migration, or `extends:` pointing to
  a non-existent file after a refactor).
- **CI lies.** The release workflow has not run in the last N days but the
  README claims the registry is live. Or the reverse: CI is green but the
  registry is empty.
- **Unresolvable pin.** A benchmark's `eval.benchmark.data_revision` points at a
  sha that returns 404 upstream.

### Yellow signals (worth attention, not blocking)

- **Stale agent.** An agent whose `eval.agent.version` is more than 90 days
  behind the project's latest release on GitHub / npm / PyPI.
- **Missing per-benchmark README.** A benchmark directory with no `README.md`.
- **Unreleased benchmark.** A directory committed but not yet pushed to the
  registry. Might be intentional or forgotten.
- **Orphan fixture.** A fixture referencing a benchmark-agent combination no
  longer in the repo.
- **Model coverage gap.** A benchmark never fixture-tested against a specific
  model axis.
- **Replay coverage gap.** A benchmark with fixtures for one agent but none of
  the others, so regressions in those combinations are silent.

### Green signals (all must be present for a `green` fleet verdict)

- **Every benchmark builds.** Full `cargo test --test build -- --ignored` sweep
  passes.
- **Every Dockerfile is healthy.** The Dockerfile rule catalog passes with zero
  red findings.
- **Every trajectory is healthy.** The trajectory rule catalog passes on every
  existing fixture.
- **Counts match docs.** README benchmark/agent/model counts match the
  filesystem.
- **All released benchmarks have fixtures.**
- **All CI workflows green on `main`.**

## Classification rules

```
if any red signal:     fleet verdict = red     # do not ship
elif any yellow:       fleet verdict = yellow  # ship with known gaps documented
else:                  fleet verdict = green   # clean release
```

Mapped to the ten questions (per `tests/fleet/RULES.md:4`): any
**no** on questions 1–5 or 9 is red; any **no** on 6–8 is yellow; question 10 is
informational but mandatory before shipping. A build failure that is inside
`tests/build/known-broken.md` is yellow, not red.

## Layered checking

**Layer 1 — mechanical counts and cross-references.** Scripted checks: walk
`benchmarks/` and `agents/`, count, compare against the README, diff against
`tests/replay/fixtures/`, compare label values with directory names, walk the
Dockerfile `FROM` graph and check every referenced image exists. Deterministic
and cheap — a shell script or a Rust test.

**Layer 2 — procedural audit.** The ten-question walk in `SKILL.md`. Covers the
subjective drift questions: are these benchmarks still what we claim? Is the
agent roster still representative of the field? Have any datasets silently
changed upstream?

**Layer 3 — upstream verification (network required).** For every pinned dataset
revision, fetch the manifest from the upstream registry and verify it still
resolves; for every agent's upstream package, query the registry for the latest
stable version and compare against the pin. Not a CI gate — needs network and is
slow — but a valuable pre-release check. This is the `upstream` test category
(`tests/upstream/RULES.md`).

## Output format

A single markdown report:

```
# Fleet Health Report — YYYY-MM-DD

## Counts
- benchmarks: 96 on disk, 96 claimed in README ✓
- agents: 17 on disk, 17 claimed in README ✓
- fixtures: 23 ✓

## Mechanical gates
- check structure: ✓
- compose: ✓ (96/96)
- dockerfile: ✓
- trajectory: ✓ (23/23 green, 0 yellow)
- build: ⚠ (4 known failures: swe-bench-pro, cybench, mle-bench, swe-lancer — upstream TODO)
- replay: ✓ (23/23)

## Fleet questions
| # | Q                          | Verdict | Notes |
|---|----------------------------|---------|-------|
| 1 | Every bench has both files | ✓       |       |
| 2 | Every bench builds         | ⚠       | 4 TODO |
| ...                                                  |

## Suggested fixes
1. Fix the per-task benchmarks' upstream image references
2. Add fixtures for the benchmarks currently uncovered
3. ...

## Verdict
⚠ yellow — ship-ready with 4 known build gaps documented
```

## When to run

- Before cutting a release (mandatory).
- Quarterly health check (recommended).
- After a batch of ≥10 new benchmarks / agents lands.
- After a RULES change that touches multiple files.

## References

- `../RULES.md` (top-level repo rules) — normative principles for the whole
  repo.
- `.agents/verification/audit-trajectory/references/checklist.md` —
  per-fixture runtime health spec.
- `.agents/verification/audit-dockerfile/references/checklist.md` —
  per-Dockerfile static health spec.
- `tests/fleet/RULES.md` — the aggregator rules.
- `.agents/delivery/RULES.md` — how CI builds and publishes the fleet.
