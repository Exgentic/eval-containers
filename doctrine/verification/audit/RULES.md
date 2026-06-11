# Audit Report Rules

**Status:** Active
**Date:** June 2026

## Abstract

Every benchmark carries an `AUDIT.md` recording what has actually been checked
about it — validity, safety, size, speed, cost, and distribution — and the root carries
a generated `AUDIT.md` aggregating each benchmark's bottom line. This document
fixes what an audit report must contain, what its statuses mean, and the rollup
over them.

## Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in
this document are to be interpreted as described in BCP 14, RFC 2119 and
RFC 8174. An *audit report* is the `AUDIT.md` file beside a benchmark's
`Dockerfile`. A *check* is one audited property; a *status* is a check's
recorded outcome.

## Requirements

1. **Present.** Every released benchmark **MUST** ship an `AUDIT.md` beside its
   `Dockerfile`.

2. **Provenance.** An `AUDIT.md` **MUST** open with frontmatter giving its
   benchmark, host, and audit commit.

3. **Dimensions.** An `AUDIT.md` **MUST** cover validity, safety, size, speed,
   cost, and distribution.

4. **Vocabulary.** Every check **MUST** carry one status: verified, failing,
   unchecked, or not-applicable.

5. **Verified means checked.** A verified status **MUST** mean a check passed,
   not a property true by construction.

6. **Evidence.** A verified or failing status **MUST** cite its evidence.

7. **Honest gaps.** An unchecked property **MUST** be recorded as unchecked, not
   omitted.

8. **Reproducible.** A check a tool can measure **MUST** record a value matching
   that tool's output.

9. **Rollup.** An `AUDIT.md` in `containers/` (the fleet root) **MUST** summarize every
   benchmark's audit, one row each.

10. **Generated.** The `containers/AUDIT.md` rollup **MUST** be generated from the per-benchmark
    reports, not hand-edited.

11. **Stale-aware.** The `containers/AUDIT.md` rollup **MUST** mark a benchmark whose sources
    changed after its audit's commit as stale.

## References

- `doctrine/meta/rules/RULES.md` — the form of this document.
- `doctrine/verification/audit/audit-benchmark/SKILL.md` — the procedure that
  produces a benchmark's `AUDIT.md`.
- `doctrine/verification/audit/audit-rollup/SKILL.md` — the procedure that
  generates `containers/AUDIT.md`.
- `core/oracle/README.md` — the oracle behind the validity `oracle` check.

## Changelog

| Date       | Change           |
|------------|------------------|
| 2026-06-10 | Initial version. |
