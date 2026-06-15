# Eval Containers Rules

**Status:** Active
**Date:** April 2026

## Abstract

Eval Containers is a build system for AI agent evaluations. It produces Docker images and Compose files. This document defines the core principles of the project and how rules work.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in all RULES documents are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Core Principles

1. **The image is the product.** Everything Eval Containers produces is a Docker image or Compose file. Images are immutable, versioned, portable. If you can `docker pull` and `docker compose up`, you can run any evaluation.

2. **Standalone artifacts.** Every published image and Compose file MUST work without Eval Containers installed. If the Eval Containers repository is deleted, every artifact MUST still work.

3. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file. One format for simple and complex benchmarks alike.

4. **Three independent axes.** An evaluation is one benchmark + one agent + one model. Each MUST be swappable without affecting the others.

5. **Independent observation.** All LLM calls MUST be logged by the model service, independent of the agent. The agent MUST NOT know the proxy exists. The agent MUST NOT be able to tamper with the trajectory.

6. **No framework lock-in.** Eval Containers MUST NOT require its own runtime, daemon, or installation to execute evaluations. Everything runs with plain Docker.

7. **Simplicity.** Prefer the simplest mechanism that works. File permissions over separate containers. Shell scripts over frameworks. Flat files over databases. If it can be a one-liner, it should be.

8. **Clean code.** All code MUST be the most simple, minimal, clean, and easy to maintain implementation that serves its goals. No dead code, no premature abstractions, no unnecessary dependencies. This is not a suggestion.

9. **Pin by default, control via two orthogonal knobs.** Every benchmark, agent, and model image MUST ship with a reproducible default that runs with no environment variables set. Every image MUST expose two independent version controls:

   - **Container version** (the Eval Containers-authored wiring) is selected by the **image tag**, set via `EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`. This is Docker's native versioning mechanism — different tag, different pull, different bits. The fleet-wide default is **one release version**: a SemVer set by the git tag (`latest` on `main`), applied to every image and to the per-benchmark `eval-<benchmark>` compose + `charts/eval` artifacts. `Cargo.toml` and the Helm `Chart.yaml` MUST carry that same version (guard: `tests/static/check.rs`). Bumps: **major** = breaking (a benchmark removed or renamed, the `EVAL_*` contract or `result.json`/output format changed); **minor** = additive (new benchmarks/agents/models, new backward-compatible flags); **patch** = rebuilds, base-image/CVE updates, fixes with no behavior change. The per-component `EVAL_*_TAG` overrides pull a single artifact at a different release version. The tag encodes *our* version, never the upstream software version (that is the Internal version below).

   - **Internal version** (the upstream software baked or installed inside) is selected at runtime via `EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`. The framework launcher (`/usr/local/bin/run`) MUST read these env vars, install or activate the requested version, and write the resolved version to the run output directory so every run record is self-describing.

   Both axes are orthogonal: tag controls which container to pull, env var controls what runs inside it. Concrete implementation rules live in `.agents/benchmarks/RULES.md`, `.agents/agents/RULES.md`, and `.agents/models/RULES.md`.

10. **Image hygiene.** Eval Containers ships ~100+ images. Image thinness is not optional — it's what makes `docker pull` fast and the fleet build affordable. Every Dockerfile MUST follow these rules, and reviewers MUST enforce them:

    a. **Slim bases.** Prefer `python:3.12-slim` over `python:3.12`. Prefer `debian:12-slim` or `ubuntu:24.04` over their full variants. Avoid `alpine` for Python workloads (musl wheels missing). `FROM scratch` only for static binaries.

    b. **Clean up in the same layer.** `apt-get install` MUST be followed by `rm -rf /var/lib/apt/lists/*` in the same `RUN`. `pip install` MUST use `--no-cache-dir`. Cleanup in a separate `RUN` does nothing — the prior layer still holds the files.

    c. **No caches or secrets in layers.** Never `COPY` a `~/.cache`, `~/.npm`, `~/.cargo`, credentials, or any build cache directory into a final image. Use `--mount=type=secret` when auth is needed at build time.

    d. **Simple and impactful, not clever.** Do the obvious cleanups. Do NOT rewrite Dockerfiles into multi-stage contortions to save 20 MB. The rule of thumb: if a one-line diff saves ≥100 MB, do it; if it saves less but hurts readability, don't. Maintainability wins over byte-golfing.

    e. **No required rebuild on `docker history`.** A reviewer MUST be able to read the Dockerfile top-to-bottom and see why each layer exists. Reordering layers for optimal cache hits is fine; obscuring them for layer count is not.

11. **Reuse over repetition.** Any infrastructure concern shared by more than two images MUST be factored into a shared base image or helper, not inlined. If a Dockerfile contains ten lines of apt-retry boilerplate, network-flake defense, runtime staging, or label setup that another Dockerfile also contains, that ten lines belongs in `core/<something>-base`. Consequences:

    a. **Agent base images.** Every agent image MUST extend `core/agent-base-<runtime>` (node, python, go, universal) for its runtime. The base provides: apt-retry wrapper, runtime install with arch detection, `/opt/agent/` staging scaffold, standard `install.sh` skeleton that the combination image calls.

    b. **Benchmark base images.** Every benchmark image MUST extend `core/benchmark-base-<pattern>` (hf-dataset, github-raw, per-task-upstream, external-graded). The base provides: Python + pyarrow + datasets, the shared label/ENV scaffold, a copy of `/eval-materialize-task` (and `eval-sitecustomize.py` site customization), a standard `entrypoint.sh` template, and a default `/grade.sh` for the matching grader.

    c. **One home per concern.** A retry loop for apt, a tarball-size check, an arch detection `case` statement — each MUST appear in exactly one Dockerfile (the base). If you find yourself copy-pasting defensive code from another Dockerfile, you are violating this rule; the correct fix is to move that code to the base and have both callers inherit.

    d. **Rule precedence.** Reuse wins over "keep files self-contained". A 108-line agent Dockerfile that duplicates the base is worse than a 15-line one that extends it, even if the 108-line version is readable on its own. The 10-line base + 15-line subclass is readable too, and it's readable across 20 agents instead of 1.

    e. **No drift between inlined copies.** If a fix has to be applied (e.g. "use linux-arm64 on Apple Silicon"), applying it once in the base MUST propagate to every subclass on the next rebuild. Fleets that require N copies of a fix are a code smell.

12. **Env var namespace.** All Eval Containers-controlled environment variables MUST be prefixed with `EVAL_`. This includes axis selection (`EVAL_BENCHMARK`, `EVAL_AGENT`, `EVAL_MODEL`), versioning (`EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_MODEL_VERSION`), runtime config (`EVAL_TASK_ID`, `EVAL_TIMEOUT`), and infrastructure (`EVAL_REGISTRY`). No Eval Containers env var MAY be unprefixed. Upstream env vars (`OPENAI_API_KEY`, `HF_TOKEN`, etc.) are untouched. Prefixing prevents collision with CI, Airflow, Celery, and other orchestrators that use unprefixed names like `TASK_ID`, `AGENT`, and `MODEL`.

13. **Self-contained repository.** The repository MUST be the sole source of information about itself — every rule, convention, process, assumption, and verification procedure MUST be documented inside the tree. No essential information MAY live only in a tool-specific directory (`.claude/`, `.cursor/`, `.vscode/`), in a single contributor's head, in a chat log, or in an external wiki. A reader who clones the repo on a clean machine MUST be able to build, test, verify, and release it using only the files in the tree. Tool-specific folders (`.claude/` etc.) MAY exist, but MUST contain only convenience wrappers that delegate to the canonical docs — never original, load-bearing content.

14. **Verification is normative.** Every change MUST pass the mechanical gates, and every release MUST also pass the procedural audits defined in [.agents/verification/verify/SKILL.md](verification/verify/SKILL.md). VERIFY.md is the complete release checklist: 46 numbered steps across Preflight, Sanity, Build, Replay, End-to-end, Upstream, Security, Audit, Docs, CI, Fleet, Release, and Post phases, each with its executor (`cargo test`, external tool, or human/sub-agent checklist) and artifact.

    - **Mechanical gates** (steps 4–10, 11–16, 18–22, 30, 31, 35) MUST run from plain `cargo test` with no bash glue. Every gate is a data-driven rule catalog — a `const RULES: &[Rule]` array whose IDs match the entries in its companion procedural markdown. The two MUST NOT drift.

    - **Procedural audits** (steps 23–27) MUST be walkable by any reader — human, sub-agent, or script — using only the canonical checklists ([.agents/verification/audit-dockerfile/references/checklist.md](verification/audit-dockerfile/references/checklist.md), [.agents/verification/audit-trajectory/references/checklist.md](verification/audit-trajectory/references/checklist.md), [.agents/verification/audit-fleet/references/checklist.md](verification/audit-fleet/references/checklist.md)). Checklists MUST NOT name a specific tool, agent, or runtime. A human with an editor and an AI assistant reading the same file MUST execute the same procedure.

    - **The fleet report** ([tests/run/fleet/report.md](../tests/run/fleet/report.md), generated by `cargo test --test fleet -- --ignored`) is the single artifact that certifies a commit as release-ready. It has two sections — auto-generated (mechanical) and manual (procedural) — and a verdict of red, yellow, or green. No release MAY ship with a red verdict.

15. **Build graph is data.** Every artifact under `core/`, `agents/`, `benchmarks/`, `models/`, and `gateways/` MUST ship a `docker-bake.hcl` file next to its `Dockerfile`. The bake file is the machine-readable declaration of the artifact's build dependencies — what makes the fleet's build graph an artifact in the tree rather than knowledge trapped in one consumer. Concrete shape:

    a. **One file per artifact, declaring a single target.** Target name is `<category>-<name>` (e.g. `agent-openhands`, `benchmark-aime`, `model-gpt-5_4--bifrost`); leaf `core/` images use their bare directory name (`agent-base-python`, `benchmark-base-hf`). One target per file.

    b. **`REGISTRY` and `TAG` are fleet-wide variables** declared once at the repo root (`./docker-bake.hcl`), defaulting to `ghcr.io/exgentic` and `latest` respectively. Per-artifact files MUST reference `${REGISTRY}/...:${TAG}` in every image reference (tag and context) — no hardcoded registries, no hardcoded tags, no per-artifact redeclaration.

    c. **Tag matches the framework's existing convention**: `${REGISTRY}/<category>/<name>:${TAG}`. CI/release pipelines override `TAG` at the build step; the default `latest` covers the common dev case. Per-artifact variant tags are out of scope for bake — use `--set "<target>.tags=..."` if you really need one.

    d. **Every in-repo `FROM` (and `COPY --from=`) MUST appear in the target's `contexts`**, mapping the full image reference (`${REGISTRY}/...:tag`) to the dep's bake target (`target:<name>`). This is what makes the graph explicit and consumable by every build tool (`docker buildx`, `bakah`, `oc start-build` translators, tests).

    e. **Build-time secrets go through `args` with empty defaults** (e.g. `args = { HF_TOKEN = HF_TOKEN }`). Artifacts that don't need a given secret pay no ceremony for declaring it absent.

    f. **No build logic in bake files.** Targets, contexts, args, tags — that's it. Computed values from external sources, conditional builds, dynamic target generation are out of scope. Build logic lives in the Dockerfile. Provenance labels (`org.opencontainers.image.*`) are out of scope here too — they're stamped onto the build invocation at build time, not declared per file (see src/RULES.md principle 11).

    g. **Minimal.** Every line MUST serve sub-rules a–f. No `inherits` chains. No `group` blocks. No `dockerfile-inline`. No multi-target files. No comments restating this rule or citing it. No unused variables. No `args` declared but not consumed by the Dockerfile. The framework's existing principle 8 (Clean code) applies to bake files like any other code; this sub-rule pins the specific patterns that drift.

    h. **Variable hygiene.** A `variable` declaration MUST exist for exactly one of: a **per-build override** (the value legitimately varies per invocation — `REGISTRY`, `TAG`), a **build-time secret** (the value MUST NOT live in the file as a literal — `HF_TOKEN`), or an **orchestration-composed reference** (the value is computed by the CLI / wrapper before invoking bake — `BENCHMARK_IMAGE`, `AGENT_IMAGE`, `MODEL_IMAGE` in the combination template). Values that ARE the artifact — target name, context directory, the structural shape of the `FROM` graph, the suffix `:${TAG}` itself — MUST be hardcoded. A `variable` is configuration, not a hiding place for artifact identity. Fleet-wide variables (15.b) live at root; artifact-scoped variables stay in their artifact's file. Every `variable` MUST be referenced — dead declarations fail the lint.

    The convention guide — minimal templates per artifact type, composition patterns, and the full conciseness catalog — lives in [`.agents/delivery/build/SKILL.md`](delivery/build/SKILL.md). Mechanical enforcement (every artifact has a valid bake file whose `contexts` match the Dockerfile's `FROM` lines) is `tests/build/test.rs::dockerfile_bake_alignment`.

## Process

How rules and skills are authored, formatted, placed, and kept current is
governed by the meta — [`meta/rules/RULES.md`](meta/rules/RULES.md) (the form
of rules) and [`meta/skills/RULES.md`](meta/skills/RULES.md) (the form of
skills). The complete map of every rule and skill in `.agents/` is
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
- [Security Policy](../SECURITY.md) — vulnerability reporting and supply-chain standards

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
| 2026-05-31 | Added principle 15 (Build graph is data) — every artifact MUST ship a `docker-bake.hcl` next to its Dockerfile. Renumbered Rules Process principles 15-21 → 16-22. Convention guide added at [.agents/delivery/build/SKILL.md](delivery/build/SKILL.md); mechanical enforcement deferred to `tests/build/test.rs::dockerfile_bake_alignment`. |
| 2026-05-31 | Added principle 15.g (Minimal) — bake files MUST follow the conciseness conventions in [.agents/delivery/build/SKILL.md](delivery/build/SKILL.md). Promotes the "no inherits chains, no group blocks, no dockerfile-inline, no multi-target files, no unused variables, no unconsumed args" rules from convention into normative principle. |
| 2026-05-31 | Tightened principle 15.b — `REGISTRY` and `TAG` are fleet-wide, declared once at the root `./docker-bake.hcl`; per-artifact files reference `${REGISTRY}/...:${TAG}` without redeclaring (principle 11 reuse-over-repetition applied to bake variables). Per-artifact tag overrides via `--set` if genuinely needed. |
| 2026-05-31 | Added principle 15.h (Variable hygiene) — every bake `variable` exists for a documented reason (per-build override, build-time secret, or orchestration-composed reference); artifact identity stays hardcoded; dead variables fail the lint. Codifies the implicit pattern that drove the REGISTRY / TAG hoists. |
| 2026-05-31 | Centralized governance under `.agents/`: this file keeps the project principles (1–15); the Rules Process principles moved to `meta/rules/RULES.md`, the rules graph to `AGENTS.md`, and the procedures (verify/release/audits/build) became skills under `.agents/`. Supersedes the former “rules live next to the code” principle with centralized governance. |
| 2026-06-08 | Principle 9: replaced “the entrypoint” with “the framework launcher (`/usr/local/bin/run`)” — version resolution moved from eval-entrypoint.sh to run. |
| 2026-06-10 | Principle 9: ratified the **fleet-version** model — the container-version tag's fleet-wide default is one Eval Containers release SemVer (git tag; `latest` on `main`), applied to every image + the `evaluate`/`charts/eval` artifacts, with `Cargo.toml` and the Helm `Chart.yaml` pinned to it (guard: `tests/static/check.rs`). The tag encodes our version, never upstream's; `compose/RULES.md` rule 5 ("agent version as the tag") retired to match. (Open, separate: the Internal-version runtime override — `EVAL_*_VERSION` — is described here but unimplemented since #50; re-implement or formally retire in a follow-up.) |
| 2026-06-10 | Principle 15.b: default registry changed from `quay.io/eval-containers` to `ghcr.io/exgentic` — the fleet's canonical home is now GHCR (Exgentic org). `REGISTRY` / `EVAL_REGISTRY` still select any OCI registry (principle 20); only the default moved. Swept across the bake root variable, Dockerfile `ARG`s, compose files, the chart, the CLI default, and docs. |
| 2026-06-11 | Principle 15.f: clarified that provenance labels (`org.opencontainers.image.*`) are out of scope for the per-artifact bake files — they're stamped at build time (src/RULES.md principle 11), keeping these files to targets/contexts/args/tags. No change to bake-file content. |
| 2026-06-14 | Principle 9: the fleet-version default now spans the per-benchmark `eval-<benchmark>` compose artifacts (one self-contained compose per benchmark, flattened at publish) rather than a single shared `evaluate` artifact. A published OCI compose can't carry a dynamic per-benchmark `include:` — `docker compose publish` flattens includes — so per-benchmark sidecars are baked in at publish and `run --mode compose` consumes one artifact with a single `-f`. See [delivery/RULES.md](delivery/RULES.md) rule 3. |
