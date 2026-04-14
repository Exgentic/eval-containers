# Dock Rules

**Status:** Active
**Date:** April 2026

## Abstract

Dock is a build system for AI agent evaluations. It produces Docker images and Compose files. This document defines the core principles of the project and how rules work.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in all RULES documents are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Core Principles

1. **The image is the product.** Everything Dock produces is a Docker image or Compose file. Images are immutable, versioned, portable. If you can `docker pull` and `docker compose up`, you can run any evaluation.

2. **Standalone artifacts.** Every published image and Compose file MUST work without Dock installed. If the Dock repository is deleted, every artifact MUST still work.

3. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file. One format for simple and complex benchmarks alike.

4. **Three independent axes.** An evaluation is one benchmark + one agent + one model. Each MUST be swappable without affecting the others.

5. **Independent observation.** All LLM calls MUST be logged by the model service, independent of the agent. The agent MUST NOT know the proxy exists. The agent MUST NOT be able to tamper with the trajectory.

6. **No framework lock-in.** Dock MUST NOT require its own runtime, daemon, or installation to execute evaluations. Everything runs with plain Docker.

7. **Simplicity.** Prefer the simplest mechanism that works. File permissions over separate containers. Shell scripts over frameworks. Flat files over databases. If it can be a one-liner, it should be.

8. **Clean code.** All code MUST be the most simple, minimal, clean, and easy to maintain implementation that serves its goals. No dead code, no premature abstractions, no unnecessary dependencies. This is not a suggestion.

9. **Pin versions.** All benchmarks, agents, and models MUST pin their dependencies and data sources to exact versions. The pinned version MUST be recorded in a label (`dock.benchmark.data_revision`, `dock.agent.version`, `dock.model.version`). Images MUST be reproducible.

10. **Image hygiene.** Dock ships ~100+ images. Image thinness is not optional — it's what makes `docker pull` fast and the fleet build affordable. Every Dockerfile MUST follow these rules, and reviewers MUST enforce them:

    a. **Slim bases.** Prefer `python:3.12-slim` over `python:3.12`. Prefer `debian:12-slim` or `ubuntu:24.04` over their full variants. Avoid `alpine` for Python workloads (musl wheels missing). `FROM scratch` only for static binaries.

    b. **Clean up in the same layer.** `apt-get install` MUST be followed by `rm -rf /var/lib/apt/lists/*` in the same `RUN`. `pip install` MUST use `--no-cache-dir`. Cleanup in a separate `RUN` does nothing — the prior layer still holds the files.

    c. **No caches or secrets in layers.** Never `COPY` a `~/.cache`, `~/.npm`, `~/.cargo`, credentials, or any build cache directory into a final image. Use `--mount=type=secret` when auth is needed at build time.

    d. **Simple and impactful, not clever.** Do the obvious cleanups. Do NOT rewrite Dockerfiles into multi-stage contortions to save 20 MB. The rule of thumb: if a one-line diff saves ≥100 MB, do it; if it saves less but hurts readability, don't. Maintainability wins over byte-golfing.

    e. **No required rebuild on `docker history`.** A reviewer MUST be able to read the Dockerfile top-to-bottom and see why each layer exists. Reordering layers for optimal cache hits is fine; obscuring them for layer count is not.

## Rules Process

11. **Rules are normative.** All contributions MUST comply with active RULES documents. Code that violates a rule MUST NOT be merged.

12. **Principles over implementations.** Rules describe what MUST be true, not how to achieve it.

13. **Rules live next to the code they govern.** `benchmarks/RULES.md` for benchmarks, `agents/RULES.md` for agents, and so on.

14. **No repetition.** Each rule has exactly one home. A rule that applies to every Dockerfile lives in top-level `RULES.md`, not mirrored into `benchmarks/RULES.md` and `agents/RULES.md`. If you find yourself restating an existing rule, delete your copy and link to the original. Duplication makes rules drift.

15. **Status lifecycle.** Each RULES document has a status: **Draft** (proposed, not yet enforced), **Active** (enforced), or **Superseded** (replaced, linked in changelog).

16. **Format.** Every RULES document MUST contain: Status, Date, Abstract, Terminology (RFC 2119 reference), numbered Principles, References, and Changelog.

17. **Changelog is required.** Every change to an active RULES document MUST be recorded in the changelog with date and summary.

## References

- [RFC 2119: Key words for use in RFCs](https://www.rfc-editor.org/rfc/rfc2119)
- [Contributing](CONTRIBUTING.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added principle 10 (Image hygiene) — slim bases, in-layer cleanup, no caches in layers, simple and maintainable over byte-golfing. Added principle 14 (No repetition) — each rule has exactly one home; renumbered subsequent rules. |
