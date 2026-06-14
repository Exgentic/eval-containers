# Testing Strategy

**Status:** Active
**Date:** April 2026

## Abstract

Eval Containers's product is Docker images, Compose files, and the evaluations they produce. This document defines the overall testing strategy — what testing *means* in this repo, regardless of which specific category of test is being written. Per-category rules live next door in `tests/<stage>/<category>/RULES.md` (the suite is grouped into stage crates — see rule 3).

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Two verification processes

Testing exists to answer two separate questions, triggered at different points in the lifecycle. Never conflate them.

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

3. **Tests are grouped by stage, one Cargo crate per stage.** Every test lives under `tests/<stage>/`, where `<stage>` is `static` (daemon-free, every-PR, dependency-light), `build` (image builds), or `run` (live container/cluster). A category is a subfolder within its stage (`tests/run/<category>/`; the `static` category's files sit directly under `tests/static/`) with a local `RULES.md`, its `*.rs` test file(s), and any category-local data (fixtures, known-broken manifests, reports). Each target is registered via `[[test]]` in its stage crate's `Cargo.toml`, with names unique across the workspace so `cargo test --test <name>` keeps working. Shared repo-root helpers live in `tests/support`. The `static` crate is the per-PR gate and MUST NOT depend on the testcontainers/tokio/reqwest stack; that stack belongs only to `build` and `run`.

4. **Subfolder rules are local.** A rule that applies only to build tests lives in `tests/build/RULES.md`. A rule that applies across every test category lives here.

5. **No parallel audit files.** If a rule is mechanically checkable, it lives in Rust test code. If it can only be walked, it lives as a procedural rule in the appropriate `RULES.md` with a walked-audit instruction. There are NO standalone checklist `.md` files that duplicate rule text. The old `DOCKERFILE.md` / `TRAJECTORY.md` / `FLEET.md` pattern is deprecated; their content is absorbed into the relevant subfolder's `RULES.md` and its Rust rule catalog.

## Runtime rules

6. **Container runtime tests MUST use testcontainers-rs.** Tests that BUILD, RUN, START, STOP, or otherwise materialize a container MUST go through [testcontainers-rs](https://rust.testcontainers.org/). Raw `Command::new("docker").arg("build"|"run"|"up"|...)` is forbidden for any operation that creates, starts, or removes a container or image. The library handles build-context assembly, daemon connection, lifecycle, and `Drop` cleanup.

6a. **Static validation is exempt.** Tests that only READ files — Dockerfile text, compose YAML, trajectory JSON — and never build, run, or materialize a container are NOT container runtime tests. They are linters. They MAY use any tool. `docker compose config` (YAML parse), `docker manifest inspect` (metadata-only pull check), and `curl -I` (HTTP HEAD) are all static validation.

6b. **Testcontainers-rs API gaps.** Two narrow carve-outs are permitted where testcontainers-rs 0.27 has no first-class API:
   - Reading labels off a built image via `docker image inspect` (container-level metadata only in the library).
   - Removing a built image via `docker rmi -f` (library auto-cleans containers, not images).
   Both carve-outs MUST be called out in the test file's doc comment with a reference to this rule.

6c. **Builds go through `docker buildx bake`.** Per top-level RULES.md principle 15, the framework's build graph lives in `docker-bake.hcl` files. Tests that need to materialize an image MUST shell to `docker buildx bake <target> --load` (via the helper in `tests/run/common/mod.rs`) rather than using testcontainers-rs's `GenericBuildableImage`. This keeps tests, the CLI, and any out-of-process consumer (OC in-cluster builds, bakah) on the same canonical build invocation. RUN/START/STOP of containers still goes through testcontainers-rs per rule 6 — only BUILD is exempt.

7. **No API keys in contribution verification.** The `replay` model is the only LLM backend in contribution-verification tests. Any test that reads `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or any other provider credential MUST be gated behind `#[ignore]` and live under `tests/run/live/`, NOT in any continuous-verification category.

8. **Fail loud over fail silent.** Test code MUST NOT use `|| true`, `2>/dev/null`, or any other error swallowing. Known failures are documented in explicit `known-broken.md` / `broken.json` manifests per category; undocumented failures panic the run.

## Fixture lifecycle

9. **Fixtures are immutable ground truth.** Recorded trajectories under `tests/run/replay/fixtures/` are PRODUCED by release verification's live sweep and feed contribution verification's replay sweep. Contributors MUST NOT hand-edit fixtures.

10. **Every fixture has a provenance record.** Filename convention `{benchmark}-{task-id}-{agent}.trajectory.jsonl`. A sibling `tests/run/replay/fixtures/provenance.json` records the model, timestamp, and release tag under which each fixture was captured.

11. **Broken fixtures are documented, not deleted.** `tests/run/replay/fixtures/broken.json` marks fixtures whose recorded run is known-bad (refusals, wrong answers, content filter hits, max-tokens truncation). Mechanical findings on these are reported but do NOT fail the continuous tests — they are re-recorded in the next release verification cycle.

## Known-broken manifests

12. **Every test category that can have expected failures ships a known-broken manifest.**
   - `tests/build/known-broken.md` — platform/upstream build failures (qemu segfaults, gated datasets).
   - `tests/run/replay/fixtures/broken.json` — broken recorded trajectories.
   - `tests/run/live/known-broken.md` — benchmarks that cannot run live (require secrets the release runner lacks).

   The test probe for each category MUST compare actual failures to its manifest. Any excess failure is red; failures within the manifest are yellow, not red.

## Rule precedence

13. **Mechanical > procedural > aspirational.** If the same rule can be enforced three ways, prefer the most automated one:
   - **Mechanical**: a Rust rule in a `test.rs` catalog. Runs on every `cargo test`. Preferred.
   - **Procedural**: a walked audit. Documented in `RULES.md` with a step-by-step audit procedure. Runs in release verification only.
   - **Aspirational**: prose in `RULES.md` with no mechanical check and no walked audit. Carries no weight. Discouraged.

   A rule stated only aspirationally is a comment, not a rule. If it matters, write the check.

## Layout

The verification **strategy** (this file) and the **procedures** (the `verify`
and `audit-*` skills) live in `.agents/verification/`. Each test **category**
keeps its rules beside the Rust that enforces them, grouped by **stage** under
`tests/{static,build,run}/` (the stage is also a Cargo crate; shared helpers in
`tests/support`):

- [static](../../tests/static/RULES.md) — fast mechanical gates (the *Sanity* verify phase)
- [build](../../tests/build/RULES.md) — build sweep
- [replay](../../tests/run/replay/RULES.md) — recorded-trajectory sweep
- [upstream](../../tests/run/upstream/RULES.md) — network reachability
- [live](../../tests/run/live/RULES.md) — live-inference sweep
- [fleet](../../tests/run/fleet/RULES.md) — aggregator and report
- [cli](../../tests/cli/RULES.md) — CLI unit tests
- [containers](../../tests/containers/RULES.md) — container runtime tests
- [gateways](../../tests/run/gateways/RULES.md) — gateway tests
- [agents](../../tests/run/agents/RULES.md) — agent test rules

A category's `RULES.md` is the markdown half of a catalog whose entries pair
one-to-one with the `const RULES: &[Rule]` arrays in its sibling `*.rs`; the
two MUST NOT drift. That pairing is why per-category rules stay beside their
tests rather than moving into `.agents/`.

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
| 2026-06-14 | Rule 3: split the suite into per-stage Cargo crates (`tests/static` / `tests/build` / `tests/run` + shared `tests/support`). The dependency-light `static` crate is the per-PR gate and excludes the testcontainers/tokio/reqwest stack. Test target names preserved, so `cargo test --test <name>` is unchanged. |
