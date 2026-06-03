# Eval Containers Rules

**Status:** Active
**Date:** April 2026

## Abstract

Eval Containers is a build system for AI agent evaluations. It produces Docker images and Compose files. This document defines the core principles of the project and how rules work.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in all RULES documents are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Core Principles

1. **The image is the product.** Everything Eval Containers produces MUST be a Docker image or Compose file, immutable, versioned, and portable.

2. **Standalone artifacts.** Every published image and Compose file MUST work without Eval Containers installed, even if the repository is deleted.

3. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file.

4. **Three independent axes.** An evaluation is one benchmark, one agent, and one model, each of which MUST be swappable without affecting the others.

5. **Independent observation.** All LLM calls MUST be logged by the model service independent of the agent; the agent MUST NOT know the proxy exists and MUST NOT be able to tamper with the trajectory.

6. **No framework lock-in.** Eval Containers MUST NOT require its own runtime, daemon, or installation to execute evaluations.

7. **Simplicity.** The simplest mechanism that works MUST be preferred.

8. **Clean code.** All code MUST be the simplest, minimal, clean implementation that serves its goals, with no dead code, premature abstractions, or unnecessary dependencies.

9. **Pin by default, control via two orthogonal knobs.** Every benchmark, agent, and model image MUST ship a reproducible default that runs with no environment variables set, and MUST expose two independent version controls:

   - **Container version** (the Eval Containers-authored wiring) is selected by the image tag, set via `EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`.

   - **Internal version** (the upstream software inside) is selected at runtime via `EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`; the entrypoint MUST read these, activate the requested version, and write the resolved version to the run output directory.

   Implementation rules live in `doctrine/benchmarks/RULES.md`, `doctrine/agents/RULES.md`, and `doctrine/models/RULES.md`.

10. **Image hygiene.** Every Dockerfile MUST follow these rules, and reviewers MUST enforce them:

    a. **Slim bases.** Slim base images MUST be preferred over their full variants, `alpine` MUST be avoided for Python workloads, and `FROM scratch` MUST be used only for static binaries.

    b. **Clean up in the same layer.** `apt-get install` MUST be followed by `rm -rf /var/lib/apt/lists/*` in the same `RUN`, and `pip install` MUST use `--no-cache-dir`.

    c. **No caches or secrets in layers.** A cache directory, credential, or build cache MUST NOT be copied into a final image; `--mount=type=secret` MUST be used when build-time auth is needed.

    d. **Simple and impactful, not clever.** Maintainability MUST win over byte-golfing.

    e. **Readable layers.** A reviewer MUST be able to read the Dockerfile top-to-bottom and see why each layer exists.

11. **Reuse over repetition.** Any infrastructure concern shared by more than two images MUST be factored into a shared base image or helper, not inlined.

    a. **Agent base images.** Every agent image MUST extend `core/agent-base-<runtime>` for its runtime.

    b. **Benchmark base images.** Every benchmark image MUST extend `core/benchmark-base-<pattern>`.

    c. **One home per concern.** Each piece of shared defensive code MUST appear in exactly one Dockerfile, the base.

    d. **Rule precedence.** Reuse MUST win over keeping files self-contained.

    e. **No drift between inlined copies.** A fix applied once in the base MUST propagate to every subclass on the next rebuild.

12. **Env var namespace.** Every Eval Containers-controlled environment variable MUST be prefixed with `EVAL_`, and no Eval Containers env var MAY be unprefixed; upstream env vars are untouched.

13. **Self-contained repository.** The repository MUST be the sole source of information about itself, with every rule, convention, process, assumption, and verification procedure documented inside the tree; a reader who clones it on a clean machine MUST be able to build, test, verify, and release it using only the files in the tree, and tool-specific folders MUST contain only convenience wrappers that delegate to the canonical docs.

14. **Verification is normative.** Every change MUST pass the mechanical gates and every release MUST also pass the procedural audits defined in [doctrine/verification/verify/SKILL.md](verification/verify/SKILL.md).

    - **Mechanical gates** MUST run from plain `cargo test` with no bash glue, each as a data-driven `const RULES: &[Rule]` catalog whose IDs match its companion procedural markdown, and the two MUST NOT drift.

    - **Procedural audits** MUST be walkable by any reader using only the canonical checklists ([doctrine/verification/audit-dockerfile/references/checklist.md](verification/audit-dockerfile/references/checklist.md), [doctrine/verification/audit-trajectory/references/checklist.md](verification/audit-trajectory/references/checklist.md), [doctrine/verification/audit-fleet/references/checklist.md](verification/audit-fleet/references/checklist.md)), which MUST NOT name a specific tool, agent, or runtime.

    - **The fleet report** ([tests/fleet/report.md](../tests/fleet/report.md)) is the single artifact certifying a commit as release-ready; no release MAY ship with a red verdict.

15. **Build graph is data.** Every artifact under `core/`, `agents/`, `benchmarks/`, `models/`, and `gateways/` MUST ship a `docker-bake.hcl` file next to its `Dockerfile` declaring its build dependencies.

    a. **One file per artifact, declaring a single target.** The target name MUST be `<category>-<name>`, or the bare directory name for leaf `core/` images, with one target per file.

    b. **`REGISTRY` and `TAG` are fleet-wide variables** declared once at the repo root (`./docker-bake.hcl`); per-artifact files MUST reference `${REGISTRY}/...:${TAG}` in every image reference without hardcoding or redeclaring them.

    c. **Tag matches the framework's convention** `${REGISTRY}/<category>/<name>:${TAG}`; per-artifact variant tags are out of scope.

    d. **Every in-repo `FROM` and `COPY --from=` MUST appear in the target's `contexts`**, mapping the full image reference to the dep's bake target.

    e. **Build-time secrets go through `args` with empty defaults.**

    f. **No build logic in bake files**; only targets, contexts, args, and tags are permitted.

    g. **Minimal.** Every line MUST serve sub-rules a–f, with no `inherits` chains, `group` blocks, `dockerfile-inline`, multi-target files, restating comments, unused variables, or args not consumed by the Dockerfile.

    h. **Variable hygiene.** A `variable` MUST exist only for a per-build override, a build-time secret, or an orchestration-composed reference; artifact identity MUST be hardcoded, and every `variable` MUST be referenced.

    The convention guide lives in [`doctrine/delivery/build/SKILL.md`](delivery/build/SKILL.md). Mechanical enforcement is `tests/build/test.rs::dockerfile_bake_alignment`.

## Process

How rules and skills are authored, formatted, placed, and kept current is
governed by the meta — [`meta/rules/RULES.md`](meta/rules/RULES.md) (the form
of rules) and [`meta/skills/RULES.md`](meta/skills/RULES.md) (the form of
skills). The complete map of every rule and skill in `doctrine/` is
[`AGENTS.md`](../AGENTS.md).

## Issue vocabulary

Every incoming issue is one of seven types,
not free-form. The taxonomy reflects the dual axes of this repo:

1. **Rule-code drift** — a rule says X, the code does Y. The rule
   is correct; the code needs to catch up. Highest-signal report,
   keeps the rules graph honest. Template 01.
2. **Rule change proposal (RFC)** — a rule is wrong, stale, or
   counterproductive. Propose a specific change to the normative
   document with rationale, impact analysis, and migration path.
   Template 02.
3. **Bug** — behavior is wrong but no rule is being violated.
   Something just doesn't work as intended. Stand-alone, no rule
   citation required. Template 03.
4. **New benchmark request** — propose a new benchmark to add to
   the fleet. Template 04.
5. **New agent request** — propose a new agent to add to the
   fleet. Template 05.
6. **New canonical model request** — propose a new canonical model.
   Template 06.
7. **Known-broken entry** — housekeeping: document something that
   IS currently broken under a specific condition, when the fix is
   not immediate and we want the break visible and tracked. Feeds
   one of the `known-broken.md` / `broken.json` manifests in the
   testing strategy subtree. Template 07.

Questions and open-ended discussions belong in GitHub Discussions,
not as issues. The issue tracker is for tracked work only.

## References

- [RFC 2119: Key words for use in RFCs](https://www.rfc-editor.org/rfc/rfc2119)
- [Contributing](../CONTRIBUTING.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added principle 10 (Image hygiene) — slim bases, in-layer cleanup, no caches in layers, simple and maintainable over byte-golfing. Added principle 14 (No repetition) — each rule has exactly one home; renumbered subsequent rules. |
| 2026-04-14 | Rewrote principle 9: pin by default, expose version control via `EVAL_*_VERSION` env vars. Tags encode Eval Containers component version; upstream versions live in labels, env vars, and run records. Added principle 11 (Env var namespace) — all Eval Containers env vars MUST be prefixed with `EVAL_` to prevent collision with CI/orchestrator env vars. Renumbered Rules Process principles (12–18). |
| 2026-04-14 | Principle 9 refined to two orthogonal knobs: container version via image tag (`EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`) and internal upstream version via runtime env var (`EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`). Models now covered by principle 9 (LiteLLM version is the internal axis). |
| 2026-04-15 | Added principle 12 (Self-contained repository) and principle 13 (Verification is normative). Added the "rules graph" section rooting every normative document in this file. Renumbered Rules Process principles (14–20). |
| 2026-04-16 | Rewrote the rules graph to reflect the tests/ subfolder restructure (sanity/build/replay/upstream/live/fleet/cli each with its own RULES.md), added PR templates as contribution entry points, and clarified the contribution-vs-release duality: same rules, different walkers. |
| 2026-04-16 | Added model PR template (`.github/PULL_REQUEST_TEMPLATE/model.md`) and seven issue templates covering the repo's seven tracked issue types: rule-code drift, rule change RFC, bug, new-benchmark/agent/model requests, known-broken entry. New "Issue vocabulary" section in RULES.md documents the taxonomy. |
| 2026-05-31 | Added principle 15 (Build graph is data) — every artifact MUST ship a `docker-bake.hcl` next to its Dockerfile. Renumbered Rules Process principles 15-21 → 16-22. Convention guide added at [doctrine/delivery/build/SKILL.md](delivery/build/SKILL.md); mechanical enforcement deferred to `tests/build/test.rs::dockerfile_bake_alignment`. |
| 2026-05-31 | Added principle 15.g (Minimal) — bake files MUST follow the conciseness conventions in [doctrine/delivery/build/SKILL.md](delivery/build/SKILL.md). Promotes the "no inherits chains, no group blocks, no dockerfile-inline, no multi-target files, no unused variables, no unconsumed args" rules from convention into normative principle. |
| 2026-05-31 | Tightened principle 15.b — `REGISTRY` and `TAG` are fleet-wide, declared once at the root `./docker-bake.hcl`; per-artifact files reference `${REGISTRY}/...:${TAG}` without redeclaring (principle 11 reuse-over-repetition applied to bake variables). Per-artifact tag overrides via `--set` if genuinely needed. |
| 2026-05-31 | Added principle 15.h (Variable hygiene) — every bake `variable` exists for a documented reason (per-build override, build-time secret, or orchestration-composed reference); artifact identity stays hardcoded; dead variables fail the lint. Codifies the implicit pattern that drove the REGISTRY / TAG hoists. |
| 2026-05-31 | Centralized governance under `doctrine/`: this file keeps the project principles (1–15); the Rules Process principles moved to `meta/rules/RULES.md`, the rules graph to `AGENTS.md`, and the procedures (verify/release/audits/build) became skills under `doctrine/`. Supersedes the former “rules live next to the code” principle with centralized governance. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
