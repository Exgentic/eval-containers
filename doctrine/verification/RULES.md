# Testing Strategy

**Status:** Active
**Date:** April 2026

## Abstract

Eval Containers's product is Docker images, Compose files, and the evaluations they produce. This document defines the overall testing strategy — what testing *means* in this repo, regardless of which specific category of test is being written. Per-category rules live next door in `tests/<category>/RULES.md`.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Two verification processes

Testing answers two separate questions, triggered at different lifecycle points, which MUST NOT be conflated.

1. **Contribution verification** — triggered on every PR. MUST pass before merging to main.
   - MUST run offline
   - MUST NOT require API keys
   - MUST complete under 2 hours total (sanity + build + replay)
   - MUST be fully reproducible by any contributor on a clean clone
   - MUST NOT include audits, live inference, or human inspection

2. **Release verification** — triggered before cutting a release tag. MUST pass before tagging.
   - MUST include every contribution verification gate
   - MUST include the **live fleet sweep**: every buildable benchmark, ≥3 tasks each, against the live model of record
   - MUST include procedural audits (Dockerfile, trajectory, fleet)
   - MUST include the upstream reachability check
   - MAY take hours; runs rarely

The **procedure** for executing each process — exact commands, order, gates — lives in [VERIFY.md](verify/SKILL.md). The procedure cites rule IDs from this file and its siblings; it does not restate them.

## Test category organization

3. **One subfolder per test category.** Every test MUST live under `tests/<category>/` with a local `RULES.md`, one or more `*.rs` integration test files, and any category-local data, registered via `[[test]]` in `Cargo.toml`.

4. **Subfolder rules are local.** A rule applying only to one test category MUST live in that category's `RULES.md`, and a rule applying across every category MUST live here.

5. **No parallel audit files.** A mechanically checkable rule MUST live in Rust test code and a walkable-only rule MUST live as a procedural rule with a walked-audit instruction, and there MUST be no standalone checklist `.md` files that duplicate rule text.

## Runtime rules

6. **Container runtime tests MUST use testcontainers-rs.** Any test that builds, runs, starts, stops, or otherwise materializes a container MUST go through [testcontainers-rs](https://rust.testcontainers.org/), and raw `docker` commands that create, start, or remove a container or image are forbidden.

6a. **Static validation is exempt.** A test that only reads files and never materializes a container is a linter, not a container runtime test, and MAY use any tool.

6b. **Testcontainers-rs API gaps.** Reading labels via `docker image inspect` and removing a built image via `docker rmi -f` are permitted carve-outs, each of which MUST be called out in the test file's doc comment with a reference to this rule.

6c. **Builds go through `docker buildx bake`.** Per top-level RULES.md principle 15, a test that materializes an image MUST shell to `docker buildx bake <target> --load` via the helper in `tests/common/mod.rs` rather than testcontainers-rs's `GenericBuildableImage`, while container RUN/START/STOP still goes through testcontainers-rs per rule 6.

7. **No API keys in contribution verification.** The `replay` model MUST be the only LLM backend in contribution-verification tests, and any test reading a provider credential MUST be gated behind `#[ignore]` and live under `tests/live/`.

8. **Fail loud over fail silent.** Test code MUST NOT swallow errors, and known failures MUST be documented in per-category `known-broken.md` / `broken.json` manifests while undocumented failures panic the run.

## Fixture lifecycle

9. **Fixtures are immutable ground truth.** Contributors MUST NOT hand-edit the recorded trajectories under `tests/replay/fixtures/`.

10. **Every fixture has a provenance record.** Each fixture MUST follow the filename convention `{benchmark}-{task-id}-{agent}.trajectory.jsonl`, and `tests/replay/fixtures/provenance.json` MUST record the model, timestamp, and release tag under which it was captured.

11. **Broken fixtures are documented, not deleted.** A known-bad recorded run MUST be marked in `tests/replay/fixtures/broken.json`, and mechanical findings on it MUST NOT fail the continuous tests.

## Known-broken manifests

12. **Every test category that can have expected failures ships a known-broken manifest.** Each category's test probe MUST compare actual failures to its manifest, treating any excess failure as red and failures within the manifest as yellow.
   - `tests/build/known-broken.md` — platform/upstream build failures.
   - `tests/replay/fixtures/broken.json` — broken recorded trajectories.
   - `tests/live/known-broken.md` — benchmarks that cannot run live.

## Rule precedence

13. **Mechanical > procedural > aspirational.** A rule enforceable more than one way MUST be enforced by the most automated one available — a mechanical Rust catalog rule over a procedural walked audit over aspirational prose.

## Layout

The verification **strategy** (this file) and the **procedures** (the `verify`
and `audit-*` skills) live in `doctrine/verification/`. Each test **category**
keeps its rules beside the Rust that enforces them, under `tests/<category>/`:

- [sanity](../../tests/sanity/RULES.md) — fast mechanical gates
- [build](../../tests/build/RULES.md) — build sweep
- [replay](../../tests/replay/RULES.md) — recorded-trajectory sweep
- [upstream](../../tests/upstream/RULES.md) — network reachability
- [live](../../tests/live/RULES.md) — live-inference sweep
- [fleet](../../tests/fleet/RULES.md) — aggregator and report
- [cli](../../tests/cli/RULES.md) — CLI unit tests
- [containers](../../tests/containers/RULES.md) — container runtime tests
- [gateways](../../tests/gateways/RULES.md) — gateway tests
- [agents](../../tests/agents/RULES.md) — agent test rules

A category's `RULES.md` is the markdown half of a catalog whose entries pair
one-to-one with the `const RULES: &[Rule]` arrays in its sibling `*.rs`; the
two MUST NOT drift. That pairing is why per-category rules stay beside their
tests rather than moving into `doctrine/`.

## References

- [Top-level process rules](../RULES.md)
- [the verify skill](verify/SKILL.md) — the procedure that executes these rules
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
| 2026-04-15 | Narrow rule 2 to runtime tests; add carve-out 2a for static validation |
| 2026-04-15 | Rewrite as testing strategy. Two verification processes; subfolder organization; known-broken manifests; fixture provenance; mechanical > procedural > aspirational precedence. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
