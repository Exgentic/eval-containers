---
name: audit-dockerfile
description: >-
  Run a judgment-level health review of one or more benchmark/agent Dockerfiles
  — the procedural layer that catches what the mechanical Dockerfile rule catalog
  cannot: unreasonable install order, dead code, image bloat, labels that
  misdescribe the image, unsafe entrypoints, and subtle smells a reviewer would
  flag on sight. Use this for "audit the Dockerfiles", when the mechanical
  catalog flags a yellow, when a new benchmark/agent batch lands, or as step 23
  of the `verify` release walk. This is the per-file Dockerfile pass; for
  per-fixture trajectories use audit-trajectory, and for the whole-repo sweep use
  audit-fleet.
---

# Audit a Dockerfile's health

Structural validation tells you a Dockerfile is **present**. `docker build`
tells you it **compiles**. Neither tells you it is **sane** — no hardcoded
secrets, no version drift, no convention violations, no TODOs, no subtle
mistakes a reviewer would flag. This audit is the procedural layer that closes
that gap. It applies to a single Dockerfile, a batch, or the whole fleet, and it
is toolchain-agnostic: a human reads the file in their editor, or a sub-agent
reads this checklist and executes it — one sub-agent per Dockerfile, verdicts
collected. The output format is fixed so findings are comparable across
releases.

The full red / yellow / green signal catalog and the layered (mechanical →
procedural → external-linter) model are bulky reference material; they live in
`references/checklist.md` beside this skill. Read it before walking the seven
questions so you know what the mechanical layer already covers.

## Rules this skill serves

- `tests/sanity/RULES.md:5` — the data-driven Dockerfile rule
  catalog (one ID, severity, why, and predicate per rule) is the mechanical
  layer this audit sits on top of; the audit finds what those rules miss.
- `tests/sanity/RULES.md:9` — prefer mechanical over procedural:
  any pattern this walk surfaces that *could* be a text check should be proposed
  as a new catalog rule, not left as a recurring manual finding.
- `doctrine/verification/RULES.md:13` — mechanical > procedural > aspirational;
  this is the procedural tier, run in release verification, not a substitute for
  the mechanical catalog.
- `tests/agents/RULES.md` and `benchmarks/RULES.md` — the
  required-label sets (agent `eval.agent.version`, benchmark `eval.benchmark.*`)
  that question 5 checks the image against.

## Procedure

For each Dockerfile under `benchmarks/*/` or `agents/*/`:

1. **Gather context.** Note the parent directory name (for label-consistency
   checks), its expected type (benchmark or agent), and its sibling files
   (`compose.yaml`, `install.sh`, `entrypoint.sh`). WHY: most findings — label
   drift, a version that does not match the install command, a heavy package no
   sibling script uses — are only visible against this context.

2. **Run the mechanical catalog first.** `cargo test --test check dockerfile`.
   Note what it found. WHY: the audit's job is to find what the rules *missed*,
   not to duplicate them (`tests/sanity/RULES.md:5`).

3. **Read the Dockerfile end to end** and answer the seven questions, marking
   each yes / no / n.a. with a one-line reason. WHY: each question targets a
   class of defect the regex catalog cannot judge.

   | # | Question |
   |---|----------|
   | 1 | Does the install sequence make sense? (base → system deps → language runtime → app → entrypoint) |
   | 2 | Are comments sufficient — could a new maintainer understand WHY each layer exists? |
   | 3 | Is there dead code (unused ARGs, dangling COPY destinations, commented-out blocks)? |
   | 4 | Does the image include anything the runtime does not need (build toolchains, docs, sample data)? |
   | 5 | Does the label set correctly describe the image content? (agent version matches the install command, dataset revision matches the fetch URL, etc.) |
   | 6 | Is the entrypoint sane — reads the right env vars, handles missing defaults, exits cleanly? |
   | 7 | Any subtle smells a reviewer would flag but the rules did not catch? |

4. **Classify the verdict.** WHY: questions 5 and 6 are correctness — a wrong
   answer there means the image is actively wrong, not merely untidy.
   - **healthy** — all seven answers are yes or n.a.
   - **needs attention** — any question is no but the image still works.
   - **broken** — question 5 or 6 is no (the image is wrong).

5. **Emit one report entry per Dockerfile** in the fixed format, then a summary
   count and the top suggested fixes. WHY: the fixed shape lets findings diff
   cleanly across releases.

   ```
   ## benchmarks/aime/Dockerfile
   - Mechanical rules: ✓ (0 findings)
   - Q1 (install order): ✓
   - Q2 (comments): ✓
   - Q3 (dead code): ✓
   - Q4 (bloat): ⚠ ships pyarrow for build-time parquet parse, uninstalled at L23 — good
   - Q5 (labels): ✓ data_revision matches URL sha
   - Q6 (entrypoint): ✓
   - Q7 (smells): ✓
   - Verdict: healthy
   ```

6. **Propose catalog rules for any mechanical-shaped finding.** If a smell
   recurs and could be a text check (e.g. a `"TODO"` string literal inside a
   `RUN`, a silent `pip install ... || true`, a duplicate post-`FROM` `ARG`, a
   phantom `pip uninstall` in a separate layer, an `ARG`/`LABEL` version
   mismatch, a missing `data_revision` when fetching a mutable ref), record it
   as a proposed new rule for the catalog. WHY: this is the mechanical >
   procedural escalation (`tests/sanity/RULES.md:9`) — a manual
   finding that keeps recurring belongs in code.

## When to run

- Before cutting a release (whole fleet) — step 23 of the `verify` skill.
- When the mechanical Dockerfile catalog flags a yellow that needs judgment.
- When a new benchmark or agent batch lands.
- When a `RULES.md` changes (old Dockerfiles may have drifted).
- Quarterly, as a health check.

## References

- `references/checklist.md` — the full red / yellow / green signal catalog,
  classification rules, layered-checking model, and output format.
- `tests/sanity/RULES.md` — the mechanical Dockerfile rule
  catalog this audit complements.
- `doctrine/verification/audit-trajectory/SKILL.md` — the parallel per-fixture
  runtime-health audit.
