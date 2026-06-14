# Testing Strategy

**Status:** Active
**Date:** April 2026

## Abstract

Eval Containers's product is Docker images, Compose files, and the evaluations
they produce. This document defines what testing means in this repo, regardless
of which category of test is written, and the form runtime tests take.
Per-category rules live beside the checks under `tests/<category>/`.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" are to be interpreted as
described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

A *runtime test* starts, runs, or stops a container. A *linter* only reads
files. An *oracle* is a service in a compose file whose exit code reports a
test's verdict. The *suite runner* is `tests/run`, which executes every check
and aggregates results as *TAP* (the Test Anything Protocol). The *contribution
lane* runs on every pull request; the *release lane* runs before a release tag.

## Two verification processes

Testing answers two separate questions at two points in the lifecycle; a test
belongs to exactly one lane.

1. Contribution verification **MUST** pass before a pull request merges to main.

   1a. Contribution verification **MUST** run offline.

   1b. Contribution verification **MUST NOT** read a provider credential.

   1c. Contribution verification **MUST** complete within two hours.

   1d. Contribution verification **MUST** be reproducible on a clean clone.

   1e. Contribution verification **MUST** exclude audits, live inference, and human inspection.

2. Release verification **MUST** pass before a release tag is cut.

   2a. Release verification **MUST** include every contribution-verification gate.

   2b. Release verification **MUST** include a live sweep of every buildable benchmark, at least three tasks each, against the model of record.

   2c. Release verification **MUST** include the Dockerfile, trajectory, and fleet audits.

   2d. Release verification **MUST** include an upstream-reachability check.

The procedures that execute these lanes live in the `verify` and `audit-*`
skills; they cite these requirements rather than restating them.

## Test category organization

3. Every test **MUST** live under `tests/<category>/` beside a local `RULES.md`.

4. A category-specific rule **MUST** live in that category's `RULES.md`.

5. A mechanically checkable rule **MUST** be enforced by a check rather than duplicated in a standalone checklist document.

## Runtime rules

6. A runtime test **MUST** be expressed as a compose file with an oracle service whose exit code is its verdict.

   6a. A linter **MAY** use any standard tool.

   6b. *[deprecated 2026-06-14 — the testcontainers-rs API-gap carve-out is superseded by 6e.]*

   6c. A test that builds an image **MUST** invoke `docker buildx bake`.

   6d. Lifecycle and readiness ordering in a runtime test **MUST** come from compose dependency conditions.

   6e. The structure of a built image **MUST** be asserted with container-structure-test.

   6f. A runtime test **MUST** remove its containers and volumes when it finishes.

   6g. A runtime test **MUST NOT** drive container lifecycle outside compose.

7. A test that reads a provider credential **MUST** run only in the release lane.

   7a. The replay model **MUST** be the only model backend in contribution verification.

8. Test code **MUST NOT** swallow errors.

## Fixture lifecycle

9. A recorded fixture **MUST NOT** be hand-edited.

10. Every fixture **MUST** carry a provenance record.

11. A known-bad fixture **MUST** be recorded in a broken manifest rather than deleted.

## Known-broken manifests

12. Every category that can have expected failures **MUST** ship a known-broken manifest.

    12a. A category's failure probe **MUST** compare actual failures against its known-broken manifest.

## Rule precedence

13. A rule that can be enforced mechanically **MUST** be enforced by a check the suite runner executes rather than left procedural.

## Layout

The verification strategy (this file) and the procedures (the `verify` and
`audit-*` skills) live in `.agents/verification/`. Each test category keeps its
rules beside the checks that enforce them, under `tests/<category>/`:

- [sanity](../../tests/sanity/RULES.md) — fast mechanical gates
- [build](../../tests/build/RULES.md) — build sweep
- [replay](../../tests/replay/RULES.md) — recorded-trajectory sweep
- [upstream](../../tests/upstream/RULES.md) — network reachability
- [live](../../tests/live/RULES.md) — live-inference sweep
- [fleet](../../tests/fleet/RULES.md) — aggregator and report
- [cli](../../cli/tests/RULES.md) — CLI unit and integration tests (Rust)
- [containers](../../tests/containers/RULES.md) — runtime container tests
- [gateways](../../tests/gateways/RULES.md) — gateway tests
- [agents](../../tests/agents/RULES.md) — agent test rules

A category's `RULES.md` pairs with the checks in its sibling `*.bats`,
`compose.yaml`, and `*.sweep.sh` files; the two **MUST NOT** drift. The CLI is
the one exception: it keeps its Rust tests in `cli/tests/`.

## References

- [Top-level process rules](../RULES.md)
- [the verify skill](verify/SKILL.md) — the procedure that executes these rules
- [container-structure-test](https://github.com/GoogleContainerTools/container-structure-test)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
| 2026-04-15 | Narrow rule 2 to runtime tests; add carve-out 2a for static validation |
| 2026-04-15 | Rewrite as testing strategy. Two verification processes; subfolder organization; known-broken manifests; fixture provenance; mechanical > procedural > aspirational precedence. |
| 2026-06-14 | Tightening pass to meta rules 11–14 (concise, atomic, example-free): every requirement rewritten to one example-free sentence. Rule 6 revised from the testcontainers-rs mandate to the compose-oracle model; added 6d–6g (compose lifecycle/ordering/teardown, container-structure-test); 6b deprecated in place (superseded by 6e); rule 7 scoped to the release lane and the lane vocabulary (oracle, TAP, suite runner) added to Terminology. Folds the absorbed no-credentials rule into 1b. (#114) |
