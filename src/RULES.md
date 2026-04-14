# CLI

**Status:** Active
**Date:** April 2026

## Abstract

The `dock` CLI is a thin Rust wrapper around Docker and Docker Compose. It exists to save keystrokes, not to add abstractions. This document defines the design principles for the CLI.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Philosophy

1. **Optional.** The CLI MUST be optional. Everything MUST work with plain `docker` and `docker compose` commands. The CLI is a shortcut, not a dependency.

2. **Transparent.** Every `dock` command MUST map to a Docker command. The CLI SHOULD print the underlying command it runs. The user MUST be able to do the same thing without the CLI.

3. **No magic.** The CLI MUST NOT introduce abstractions beyond what Docker provides. No custom runtimes, no hidden state, no daemons. If Docker can't do it, Dock doesn't promise it.

### Implementation

4. **Rust.** The CLI is written in Rust. It MUST be a single static binary with no runtime dependencies beyond Docker.

5. **Shell out.** The CLI MUST invoke `docker` and `docker compose` as subprocesses. It MUST NOT reimplement Docker functionality. It MUST NOT use Docker client libraries when a shell command suffices.

6. **Simplest implementation.** Each command SHOULD be the shortest path to calling the right `docker` command with the right arguments. Prefer string formatting over abstractions.

### Behavior

7. **Auto-build.** When running an evaluation, the CLI SHOULD build any missing images (eval, model, agent) before starting. It MUST NOT rebuild images that already exist locally.

8. **Registry-aware.** The CLI MUST use `DOCK_REGISTRY` for all image paths. The same commands MUST work against any OCI-compliant registry, including `localhost:5000`.

9. **Local-first.** The CLI SHOULD prefer locally cached images. It MUST support `--local` for development against local compose files.

10. **Env var ↔ CLI flag parity.** Every `DOCK_*` env var MUST be exposable as a CLI flag with the same name, converted from `SCREAMING_SNAKE_CASE` to `--kebab-case` and stripped of the `DOCK_` prefix. `DOCK_BENCHMARK` → `--benchmark`, `DOCK_AGENT_VERSION` → `--agent-version`, `DOCK_TASK_ID` → `--task-id`, `DOCK_TIMEOUT` → `--timeout`. When both are set, the CLI flag MUST override the env var. The CLI's job is to translate flags into the corresponding env vars before shelling out to `docker compose`.

### Commands

11. **Build.** `dock build agent|bench|model|eval` — each MUST map to a single `docker build` call.

12. **Run.** `dock run {benchmark} --agent {name} --task-id {id}` — MUST map to `docker compose up`. MUST accept `--model`, `--benchmark-version`, `--agent-version`, `--model-version`, `--timeout`, `--local` overrides.

13. **Report.** `dock report ./output/` — MUST walk the output directory, read `result.json` files, and aggregate. MUST support `--format csv|json`.

14. **List.** `dock list benchmarks|agents|models` — MUST read Docker image labels. No separate database or index.

15. **Push.** `dock push agent|bench|model|eval` — MUST map to `docker push`.

## References

- [Process](../RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added principle 10: env var / CLI flag parity — every `DOCK_*` env var MUST be exposable as a `--kebab-case` flag; CLI flag overrides env var. Updated `dock run` (principle 12) to list the standard version/timeout flags. Renumbered commands (11–15). |
