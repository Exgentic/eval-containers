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

   - **Container version** (the Eval Containers-authored wiring) is selected by the **image tag**, set via `EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`. This is Docker's native versioning mechanism — different tag, different pull, different bits.

   - **Internal version** (the upstream software baked or installed inside) is selected at runtime via `EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`. The entrypoint MUST read these env vars, install or activate the requested version, and write the resolved version to the run output directory so every run record is self-describing.

   Both axes are orthogonal: tag controls which container to pull, env var controls what runs inside it. Concrete implementation rules live in `benchmarks/RULES.md`, `agents/RULES.md`, and `models/RULES.md`.

10. **Image hygiene.** Eval Containers ships ~100+ images. Image thinness is not optional — it's what makes `docker pull` fast and the fleet build affordable. Every Dockerfile MUST follow these rules, and reviewers MUST enforce them:

    a. **Slim bases.** Prefer `python:3.12-slim` over `python:3.12`. Prefer `debian:12-slim` or `ubuntu:24.04` over their full variants. Avoid `alpine` for Python workloads (musl wheels missing). `FROM scratch` only for static binaries.

    b. **Clean up in the same layer.** `apt-get install` MUST be followed by `rm -rf /var/lib/apt/lists/*` in the same `RUN`. `pip install` MUST use `--no-cache-dir`. Cleanup in a separate `RUN` does nothing — the prior layer still holds the files.

    c. **No caches or secrets in layers.** Never `COPY` a `~/.cache`, `~/.npm`, `~/.cargo`, credentials, or any build cache directory into a final image. Use `--mount=type=secret` when auth is needed at build time.

    d. **Simple and impactful, not clever.** Do the obvious cleanups. Do NOT rewrite Dockerfiles into multi-stage contortions to save 20 MB. The rule of thumb: if a one-line diff saves ≥100 MB, do it; if it saves less but hurts readability, don't. Maintainability wins over byte-golfing.

    e. **No required rebuild on `docker history`.** A reviewer MUST be able to read the Dockerfile top-to-bottom and see why each layer exists. Reordering layers for optimal cache hits is fine; obscuring them for layer count is not.

11. **Reuse over repetition.** Any infrastructure concern shared by more than two images MUST be factored into a shared base image or helper, not inlined. If a Dockerfile contains ten lines of apt-retry boilerplate, network-flake defense, runtime staging, or label setup that another Dockerfile also contains, that ten lines belongs in `core/<something>-base`. Consequences:

    a. **Agent base images.** Every agent image MUST extend `core/agent-base-<runtime>` (node, python, go, universal) for its runtime. The base provides: apt-retry wrapper, runtime install with arch detection, `/opt/agent/` staging scaffold, standard `install.sh` skeleton that the combination image calls.

    b. **Benchmark base images.** Every benchmark image MUST extend `core/benchmark-base-<pattern>` (hf-dataset, github-raw, per-task-upstream, external-graded). The base provides: Python + pyarrow + datasets, the shared label/ENV scaffold, copies of `/eval-entrypoint.sh` and `/eval-materialize-task`, a standard `entrypoint.sh` template, and a default `test.sh` for the matching grader.

    c. **One home per concern.** A retry loop for apt, a tarball-size check, an arch detection `case` statement — each MUST appear in exactly one Dockerfile (the base). If you find yourself copy-pasting defensive code from another Dockerfile, you are violating this rule; the correct fix is to move that code to the base and have both callers inherit.

    d. **Rule precedence.** Reuse wins over "keep files self-contained". A 108-line agent Dockerfile that duplicates the base is worse than a 15-line one that extends it, even if the 108-line version is readable on its own. The 10-line base + 15-line subclass is readable too, and it's readable across 20 agents instead of 1.

    e. **No drift between inlined copies.** If a fix has to be applied (e.g. "use linux-arm64 on Apple Silicon"), applying it once in the base MUST propagate to every subclass on the next rebuild. Fleets that require N copies of a fix are a code smell.

12. **Env var namespace.** All Eval Containers-controlled environment variables MUST be prefixed with `EVAL_`. This includes axis selection (`EVAL_BENCHMARK`, `EVAL_AGENT`, `EVAL_MODEL`), versioning (`EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_MODEL_VERSION`), runtime config (`EVAL_TASK_ID`, `EVAL_TIMEOUT`), and infrastructure (`EVAL_REGISTRY`). No Eval Containers env var MAY be unprefixed. Upstream env vars (`OPENAI_API_KEY`, `HF_TOKEN`, etc.) are untouched. Prefixing prevents collision with CI, Airflow, Celery, and other orchestrators that use unprefixed names like `TASK_ID`, `AGENT`, and `MODEL`.

13. **Self-contained repository.** The repository MUST be the sole source of information about itself — every rule, convention, process, assumption, and verification procedure MUST be documented inside the tree. No essential information MAY live only in a tool-specific directory (`.claude/`, `.cursor/`, `.vscode/`), in a single contributor's head, in a chat log, or in an external wiki. A reader who clones the repo on a clean machine MUST be able to build, test, verify, and release it using only the files in the tree. Tool-specific folders (`.claude/` etc.) MAY exist, but MUST contain only convenience wrappers that delegate to the canonical docs — never original, load-bearing content.

14. **Verification is normative.** Every change MUST pass the mechanical gates, and every release MUST also pass the procedural audits defined in [tests/VERIFY.md](tests/VERIFY.md). VERIFY.md is the complete release checklist: 46 numbered steps across Preflight, Sanity, Build, Replay, End-to-end, Upstream, Security, Audit, Docs, CI, Fleet, Release, and Post phases, each with its executor (`cargo test`, external tool, or human/sub-agent checklist) and artifact.

    - **Mechanical gates** (steps 4–10, 11–16, 18–22, 30, 31, 35) MUST run from plain `cargo test` with no bash glue. Every gate is a data-driven rule catalog — a `const RULES: &[Rule]` array whose IDs match the entries in its companion procedural markdown. The two MUST NOT drift.

    - **Procedural audits** (steps 23–27) MUST be walkable by any reader — human, sub-agent, or script — using only the canonical checklists ([tests/DOCKERFILE.md](tests/DOCKERFILE.md), [tests/TRAJECTORY.md](tests/TRAJECTORY.md), [tests/FLEET.md](tests/FLEET.md)). Checklists MUST NOT name a specific tool, agent, or runtime. A human with an editor and an AI assistant reading the same file MUST execute the same procedure.

    - **The fleet report** ([tests/fleet-report.md](tests/fleet-report.md), generated by `cargo test --test fleet -- --ignored`) is the single artifact that certifies a commit as release-ready. It has two sections — auto-generated (mechanical) and manual (procedural) — and a verdict of red, yellow, or green. No release MAY ship with a red verdict.

## Rules Process

15. **Rules are normative.** All contributions MUST comply with active RULES documents. Code that violates a rule MUST NOT be merged.

16. **Principles over implementations.** Rules describe what MUST be true, not how to achieve it.

17. **Rules live next to the code they govern.** `benchmarks/RULES.md` for benchmarks, `agents/RULES.md` for agents, `tests/*.md` for the verification procedures, and so on.

18. **No repetition.** Each rule has exactly one home. A rule that applies to every Dockerfile lives in top-level `RULES.md`, not mirrored into `benchmarks/RULES.md` and `agents/RULES.md`. If you find yourself restating an existing rule, delete your copy and link to the original. Duplication makes rules drift.

19. **Status lifecycle.** Each RULES document has a status: **Draft** (proposed, not yet enforced), **Active** (enforced), or **Superseded** (replaced, linked in changelog).

20. **Format.** Every RULES document MUST contain: Status, Date, Abstract, Terminology (RFC 2119 reference), numbered Principles, References, and Changelog.

21. **Changelog is required.** Every change to an active RULES document MUST be recorded in the changelog with date and summary.

## The rules graph

Every normative document in the repository is a node in the **rules
graph** rooted here. A reader navigates from the top-level `RULES.md`
(this file) to the per-area RULES documents, which govern their own
subdirectories, and to the verification procedures under `tests/`,
which govern how compliance is proven.

```
RULES.md  ← top-level principles (this file)
│
├── Artifact contracts (what builds and ships)
│   ├── benchmarks/RULES.md      ← per-benchmark build contract
│   │   └── benchmarks/TEMPLATE.md
│   ├── agents/RULES.md          ← per-agent build contract
│   │   └── agents/TEMPLATE.md
│   ├── models/RULES.md          ← per-model build contract
│   ├── compose/RULES.md         ← compose file contract
│   └── src/RULES.md             ← CLI surface rules
│
├── Testing strategy (how we prove compliance)
│   └── tests/RULES.md           ← two-process model + subfolder map
│       ├── tests/sanity/RULES.md    ← fast mechanical gates
│       ├── tests/build/RULES.md     ← container build sweep
│       │   └── tests/build/known-broken.md
│       ├── tests/replay/RULES.md    ← recorded-trajectory sweep
│       │   └── tests/replay/fixtures/broken.json
│       ├── tests/upstream/RULES.md  ← network reachability probe
│       ├── tests/live/RULES.md      ← live-inference sweep + trace
│       │   │                          inspection checklist
│       │   ├── tests/live/matrix.md  ← authoritative plan (generated)
│       │   └── tests/live/known-broken.md
│       ├── tests/fleet/RULES.md     ← aggregator + release report
│       │   └── tests/fleet/report.md
│       └── tests/cli/RULES.md       ← CLI unit-test rules
│
├── Procedure doc (how to execute the strategy)
│   └── tests/VERIFY.md          ← two-process procedure: which gates
│                                  run in contribution verification vs.
│                                  release verification, in what order
│
├── Contribution PR templates (walked before merge)
│   ├── .github/PULL_REQUEST_TEMPLATE.md               ← general PRs
│   ├── .github/PULL_REQUEST_TEMPLATE/benchmark.md     ← new benchmark
│   ├── .github/PULL_REQUEST_TEMPLATE/agent.md         ← new agent
│   └── .github/PULL_REQUEST_TEMPLATE/model.md         ← new canonical model
│
└── Issue templates (every incoming report picks one)
    ├── .github/ISSUE_TEMPLATE/config.yml              ← no blank issues
    ├── .github/ISSUE_TEMPLATE/01-rule-code-drift.md   ← rule right, code wrong
    ├── .github/ISSUE_TEMPLATE/02-rule-change-proposal.md ← rule wrong, propose change
    ├── .github/ISSUE_TEMPLATE/03-bug.md               ← behavior wrong, no rule violated
    ├── .github/ISSUE_TEMPLATE/04-new-benchmark-request.md
    ├── .github/ISSUE_TEMPLATE/05-new-agent-request.md
    ├── .github/ISSUE_TEMPLATE/06-new-model-request.md
    └── .github/ISSUE_TEMPLATE/07-known-broken-entry.md ← housekeeping record
```

Every rule in the graph has exactly one home (principle 17). A reader
with no prior context, reading top-down from `RULES.md`, MUST be able
to reach every other normative document in the tree by following the
links — no essential information lives outside this graph.

**Contribution vs. release entry points**. A contributor opening a
PR for a new benchmark, agent, or canonical model walks the relevant
PR template, which cites rules from the appropriate `RULES.md` and
embeds the trace inspection checklist from `tests/live/RULES.md`. A
release manager cutting a tag walks `tests/VERIFY.md`, which invokes
every mechanical gate and procedural audit listed in the testing
strategy above. The PR templates and VERIFY.md are two faces of the
same underlying rules — they differ only in *when* the checks run
and *who* walks them.

**Issue vocabulary**. Every incoming issue is one of seven types,
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
- [Contributing](CONTRIBUTING.md)

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
