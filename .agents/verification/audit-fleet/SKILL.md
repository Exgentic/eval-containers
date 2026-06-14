---
name: audit-fleet
description: >-
  Run the whole-repository health audit that a release manager reads before
  cutting a tag — run every mechanical gate, reconcile counts against the README,
  walk the ten fleet questions (does every benchmark build, does every released
  benchmark have a fixture, do the docs match disk, are agent versions pinned, is
  CI real), and classify a red / yellow / green verdict. Use this for "audit the
  fleet", "is the repo release-ready", a quarterly health check, or as step 25 of
  the `verify` release walk. This is the whole-repo cross-cutting pass; for a
  single Dockerfile use audit-dockerfile, for a single trajectory use
  audit-trajectory, and for RULES-text-vs-code drift use audit-rules-drift.
---

# Audit the fleet's health

Per-file inspections (trajectory health, Dockerfile health) tell you each
benchmark and agent is individually sound. They do not tell you the **fleet** is
sound. A fleet is sound when every documentation claim matches disk, every
benchmark has the tests it is supposed to have, every referenced agent/model
version builds, nothing has silently drifted below the quality bar, and nothing
is stale. This audit is the high-level sanity check run before a release, before
a pitch, or quarterly. It identifies gaps, regressions, and drift across the
whole repo rather than inside a single file.

The audit is the procedural layer over the `fleet` aggregator. It is
toolchain-agnostic: the mechanical gates are `cargo test`, and the ten questions
are a checklist any reviewer — human, AI assistant, or script — can walk. The
output format is fixed so reports diff across time.

The full red / yellow / green signal catalog and the three-layer checking model
are bulky reference material; they live in `references/checklist.md` beside this
skill. Read it before walking the ten questions.

## Rules this skill serves

- `tests/fleet/RULES.md:3` — the report has three sections
  (mechanical gates, procedural audits, verdict); this audit produces the
  procedural section and the verdict.
- `tests/fleet/RULES.md:4` — the verdict classification (green =
  all green, yellow = some yellows no reds, red = any red); step 4 of this walk
  applies it.
- `tests/fleet/RULES.md:6` — known-broken aware: a build failure
  inside `tests/build/known-broken.md` is yellow, not red; this audit honours
  the manifests when judging questions 2 and 6.
- `tests/build/RULES.md:6` — failures within the build
  known-broken manifest are yellow; failures outside it are red (question 2).
- `.agents/verification/RULES.md:13` — mechanical > procedural > aspirational;
  this audit runs the mechanical gates first, then applies judgment only to the
  drift questions rules cannot reach.

## Procedure

Input is the whole working tree at a given commit. Context: the release notes
(what is shipping?), the previous audit report (what was flagged last time?),
and the current RULES.

1. **Run all mechanical gates, in order, recording pass/fail for each.** WHY:
   the report's mechanical section is the spine; anything failing here is a
   finding before you reach judgment.
   ```
   cargo test --test check structure
   cargo test --test check compose -- --ignored
   cargo test --test check dockerfile -- --ignored
   cargo test --test check trajectory -- --ignored
   cargo test --test build -- --ignored      # the slow one
   cargo test --test replay -- --ignored      # end-to-end
   ```

2. **Reconcile counts.** WHY: documentation drift (a README claim that does not
   match disk) is a red signal — the cheapest one to check and the most
   embarrassing to ship.
   - `ls benchmarks/ | wc -l` vs the README claim (excluding `RULES.md` /
     `TEMPLATE.md`).
   - `ls agents/ | wc -l` vs the README claim (same exclusions).
   - `ls tests/replay/fixtures/*.trajectory.jsonl | wc -l` vs how many
     benchmarks have fixtures.

3. **Walk the ten fleet questions**, each yes / no / n.a. with a one-line
   reason. WHY: these are the cross-cutting drift questions no per-file check can
   see.

   | # | Question |
   |---|----------|
   | 1 | Does every benchmark in `benchmarks/` have both a `Dockerfile` and a `compose.yaml`? |
   | 2 | Does every committed benchmark and agent actually build? |
   | 3 | Does every benchmark labeled `eval.benchmark.released="true"` have at least one replay fixture under `tests/replay/fixtures/`? (see `benchmarks/RULES.md` 21a) |
   | 4 | Does the README's benchmark/agent/model count match the filesystem? |
   | 5 | Does every agent in `agents/` have a pinned `eval.agent.version` label — no `unpinned`, no `latest`? |
   | 6 | Are there any benchmarks whose Dockerfiles reference upstream images we no longer control or that have gone stale? |
   | 7 | Are there any orphan fixtures (files referencing benchmarks / agents no longer in the repo)? |
   | 8 | Is every published `eval-*` version tag in sync — no benchmark at eval-v2 while shared compose is still eval-v1? |
   | 9 | Do the RULES principles (esp. 9, 10, 11) hold across every benchmark and agent, or has anything drifted? |
   | 10 | Does the CI release workflow reflect reality — has it run recently, is the registry actually populated, does the README Quick Start work from scratch on a clean machine? |

4. **Classify the verdict.** WHY: the classification fixes which gaps block a
   release and which merely document it.
   - Any **no** on questions 1–5 or 9 is **red** (do not ship).
   - Any **no** on questions 6–8 is **yellow** (ship with the gap documented).
   - Question 10 is informational but **mandatory** before shipping.
   - Honour the known-broken manifests: a build failure inside
     `tests/build/known-broken.md` is yellow, not red
     (`tests/fleet/RULES.md:6`).

5. **Emit one report** in the fixed structure: a Counts section, a Mechanical
   gates section (one row per gate), a Fleet questions table, a Suggested fixes
   list, and a final Verdict line. WHY: the fixed shape lets reports diff across
   releases. Skeleton:

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
   - build: ⚠ (4 known failures, all in known-broken.md)
   - replay: ✓ (23/23)

   ## Fleet questions
   | # | Q                          | Verdict | Notes |
   |---|----------------------------|---------|-------|
   | 1 | Every bench has both files | ✓       |       |
   | 2 | Every bench builds         | ⚠       | 4 known-broken |
   | ...                                                  |

   ## Suggested fixes
   1. ...

   ## Verdict
   ⚠ yellow — ship-ready with N known gaps documented
   ```

## When to run

- Before cutting a release (mandatory) — step 25 of the `verify` skill.
- Quarterly health check (recommended).
- After a batch of ≥10 new benchmarks / agents lands.
- After a RULES change that touches multiple files.

## References

- `references/checklist.md` — the full red / yellow / green fleet signal
  catalog, classification rules, layered model, and output format.
- `tests/fleet/RULES.md` — the aggregator rules and verdict
  classification this audit produces.
- `.agents/verification/audit-dockerfile/SKILL.md` and
  `.agents/verification/audit-trajectory/SKILL.md` — the per-file audits whose
  findings feed the fleet picture.
- `.agents/verification/audit-rules-drift/SKILL.md` — the companion drift audit
  for question 9.
