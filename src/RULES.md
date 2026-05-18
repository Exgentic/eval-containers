# CLI

**Status:** Active
**Date:** April 2026

## Abstract

The `eval-containers` CLI is a thin Rust wrapper around Docker and Docker Compose. It exists to save keystrokes, not to add abstractions. This document defines the design principles for the CLI.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Philosophy

1. **Optional.** The CLI MUST be optional. Everything MUST work with plain `docker` and `docker compose` commands. The CLI is a shortcut, not a dependency.

2. **Transparent.** Every `eval-containers` command MUST map to a Docker command. The CLI SHOULD print the underlying command it runs. The user MUST be able to do the same thing without the CLI.

3. **No magic.** The CLI MUST NOT introduce abstractions beyond what Docker provides. No custom runtimes, no hidden state, no daemons. If Docker can't do it, Eval Containers doesn't promise it.

### Implementation

4. **Rust.** The CLI is written in Rust. It MUST be a single static binary with no runtime dependencies beyond Docker.

5. **Shell out.** The CLI MUST invoke `docker` and `docker compose` as subprocesses. It MUST NOT reimplement Docker functionality. It MUST NOT use Docker client libraries when a shell command suffices.

6. **Simplest implementation.** Each command SHOULD be the shortest path to calling the right `docker` command with the right arguments. Prefer string formatting over abstractions.

### Behavior

7. **Auto-build.** When running an evaluation, the CLI SHOULD build any missing images (eval, model, agent) before starting. It MUST NOT rebuild images that already exist locally.

8. **Registry-aware.** The CLI MUST use `EVAL_REGISTRY` for all image paths. The same commands MUST work against any OCI-compliant registry, including `localhost:5000`.

9. **Local-first.** The CLI SHOULD prefer locally cached images. It MUST support `--local` for development against local compose files.

10. **Env var ↔ CLI flag parity.** Every `EVAL_*` environment variable documented in the README or used by any `oci://` compose artifact MUST have a matching `--kebab-case` CLI flag derived by stripping `EVAL_` and lowercasing: `EVAL_BENCHMARK` → `--benchmark`, `EVAL_AGENT_VERSION` → `--agent-version`, `EVAL_TASK_ID` → `--task-id`, `EVAL_TIMEOUT` → `--timeout`, `EVAL_LITELLM_VERSION` → `--litellm-version`, and so on. No exceptions: if it's an env var the user can set, it MUST have a flag form. Positional shortcuts (e.g. `eval-containers run aime` accepting `aime` as the benchmark) are allowed but MUST NOT replace the corresponding `--flag`; both forms MUST work. When both a CLI flag and an env var are set, the CLI flag MUST override the env var. The CLI's sole job in `eval-containers run` is to translate every flag into its corresponding `EVAL_*` env var and shell out to the exact `docker compose -f oci://<registry>/evaluate up` command shown in the README.

### Commands

11. **Build.** `eval-containers build agent|bench|model|eval` — each MUST map to a single `docker build` call.

12. **Run.** `eval-containers run {benchmark} --agent {name} --task-id {id}` — MUST map to `docker compose up`. MUST accept both the container-tag axis (`--benchmark-tag`, `--agent-tag`, `--model-tag`) and the internal-version axis (`--benchmark-version`, `--agent-version`, `--litellm-version`), plus `--model`, `--timeout`, `--local`.

13. **Report.** `eval-containers report ./output/` — MUST walk the output directory, read `result.json` files, and aggregate. MUST support `--format csv|json`.

14. **List.** `eval-containers list benchmarks|agents|models` — MUST read Docker image labels. No separate database or index.

15. **Push.** `eval-containers push agent|bench|model|eval` — MUST map to `docker push`.

## References

- [Process](../RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added principle 10: env var / CLI flag parity — every `EVAL_*` env var MUST be exposable as a `--kebab-case` flag; CLI flag overrides env var. Updated `eval-containers run` (principle 12) to list the standard version/timeout flags. Renumbered commands (11–15). |
| 2026-04-14 | Updated `eval-containers run` (principle 12) to enumerate both axes of versioning: container tags (`--benchmark-tag`, `--agent-tag`, `--model-tag`) and internal upstream versions (`--benchmark-version`, `--agent-version`, `--litellm-version`). |
| 2026-04-14 | Tightened principle 10 (parity): every `EVAL_*` env var used anywhere in the README or in a published compose artifact MUST have a matching `--kebab-case` flag with no exceptions. Positional shortcuts are allowed but MUST NOT replace the flag form. `eval-containers run`'s job is to translate every flag to its env var and shell out to the exact docker compose command in the README. |
