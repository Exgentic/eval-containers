---
name: audit-benchmark
description: >-
  Produce or refresh one benchmark's AUDIT.md — the standing record of what has
  actually been checked about it (validity, safety, size, speed, cost). Use it
  when adding a benchmark, after changing its Dockerfile or grader, when its
  AUDIT.md is stale, or as the per-benchmark step of a fleet audit. It fills the
  machine-measurable fields from real tool runs and leaves human-judgment fields
  (running, traces-reviewed, replicate-official, cost) explicitly unchecked until
  a human does them. For the whole-repo sweep use audit-fleet; this skill drives
  the oracle (core/oracle/README.md) for the grading-soundness rung.
---

# Audit a benchmark — write its AUDIT.md

An `AUDIT.md` is only worth trusting if a `✓` means a check actually ran. This
procedure fills the auto-measurable fields from real runs and marks everything
else honestly unchecked, so the report never drifts into false confidence.
Serves `doctrine/verification/audit/RULES.md:1`–`8`.

## Steps

1. **Start from the template.** If `benchmarks/<name>/AUDIT.md` is absent, copy
   `references/template.md` to it. Starting from the template guarantees every
   dimension is present and every status starts `?` (RULES.md:3, :7).

2. **Stamp provenance.** Set the frontmatter `benchmark`, `host` (e.g. `local
   podman+Rosetta`), and `commit` (`git rev-parse --short HEAD`). The commit
   anchors *what code* was audited; its date and any staleness derive from git, so
   no separate date is stored (RULES.md:2).

3. **Build it on Rosetta.** Build the image per
   `docs/guides/podman-on-apple-silicon.md` §6 — single-`FROM` via
   `DOCKER_BUILDKIT=0 docker build`, multi-stage via native `podman build
   --platform linux/amd64 --pull=never`. Set **building** `✓`/`✗` with the build
   command as evidence (RULES.md:5, :6, :8).

4. **Oracle it.** Run `eval-containers oracle <name> [--task-id <id>]`. Set
   **oracle** `✓` iff gold=1.0 and no-op<1.0, recording both numbers as evidence;
   `n/a` for an LLM-judge or stub grader. This is the grading-soundness rung —
   it proves the grader is neither always-pass nor always-fail (core/oracle).

5. **Read isolation + safety off the artifact.** Confirm from the Dockerfile and
   compose that the benchmark does not undermine the framework's isolation:
   `/tasks` stays root-only and the task id never reaches the agent →
   **isolation** `✓`; the agent runs non-root → **agent-nonroot** `✓`; secrets
   enter only via `--mount=type=secret` → **secrets-isolated** `✓`; no agent
   egress is opened → **egress-blocked** `✓`. Each is inspectable without a run.

6. **Measure size + speed.** From the built image and the build/grade runs:
   image size (and the per-task multiplier for per-task benchmarks) → **Size**;
   build, grade, and end-to-end wall-clock → **Speed**. Copy the measured numbers
   in; never hand-type an estimate (RULES.md:8).

7. **Leave the rest honestly unchecked.** **running** (a live agent run),
   **traces-reviewed** (human reads a trajectory), **replicate-official** (a
   known model reproduces the published score), **resource-limited**, and
   **cost** need work this skill does not do. Mark each `?` (or `✗`), never `✓`,
   until that work actually happens (RULES.md:5, :7).

8. **Commit it, then refresh the rollup.** Commit `benchmarks/<name>/AUDIT.md`,
   then regenerate the root project table (the `audit-rollup` skill) so it
   reflects this change. The report is a tracked artifact, refreshed whenever the
   benchmark or its grader changes so its provenance never goes stale
   (RULES.md:1, :2, :10).
