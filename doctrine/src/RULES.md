# CLI

**Status:** Active
**Date:** April 2026

## Abstract

The `eval-containers` CLI is a reminder of the simplest standard way to run an eval, not a layer over it. Every command is a mnemonic for a plain `docker`, `kubectl`, or `oc` invocation, and running it MUST be reducible to, and able to print, that exact command. It is a thin Rust wrapper around the standard container, Kubernetes, and OpenShift tools that exists to save keystrokes, not add abstractions. This document defines the design principles for the CLI.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Underlying tools

The CLI shells out only to this fixed set of standard, user-installable tools. "Underlying tool" elsewhere in this document means one of:

| Tool | Used for |
|------|----------|
| `docker build` | per-task benchmark variants (outside the static bake graph) |
| `docker buildx bake` | the artifact build graph (top-level RULES.md principle 15); in-cluster builds use the *same* command against a `--driver kubernetes` builder |
| `docker compose` | `run --mode compose`; publishing compose artifacts |
| `docker run` | `run --mode container` |
| `docker push` | publishing images |
| `kubectl` (+ `helm template`) | `run --mode job` — Helm renders the shared chart, `kubectl apply -f -` submits it |
| `oc` | applying manifests on OpenShift (`helm template … \| oc apply -f -`); the `kubectl` superset for OpenShift login and registry routing |

Each tool MUST be a standard release the user can install and invoke themselves, adding a tool to this list MUST be recorded in the changelog, and the CLI MUST NOT reach for any tool outside it.

### Philosophy

1. **Optional.** The CLI MUST be optional, and everything it does MUST be reproducible by running the underlying tools (§Underlying tools) by hand.

2. **Transparent.** Every command that drives containers or clusters MUST map to one or more underlying-tool commands (§Underlying tools), the CLI SHOULD print the command(s) it runs, and state-changing or outward-reaching commands (`build`, `run`, `push`, `prune`) MUST support `--dry-run` that prints those commands without executing them.

3. **No magic.** The CLI MUST NOT introduce abstractions beyond what the underlying tools provide, and any artifact dependency graph MUST be expressed as data the tool executes rather than ordering logic re-derived inside the CLI.

### Implementation

4. **Rust.** The CLI MUST be a single static Rust binary with no runtime dependencies beyond the tools it shells out to.

5. **Shell out.** The CLI MUST invoke the underlying tools (§Underlying tools) as subprocesses, MUST NOT reimplement their functionality, and MUST NOT use their client libraries when a shell command suffices.

6. **Simplest implementation.** Each command SHOULD be the shortest path to calling the right underlying tool with the right arguments.

### Behavior

7. **Auto-build.** When running an evaluation, the CLI SHOULD build any missing images before starting and MUST NOT rebuild images that already exist locally.

8. **Registry-aware.** The CLI MUST use `EVAL_REGISTRY` for all image paths, and the same commands MUST work against any OCI-compliant registry.

9. **Local-first.** The CLI SHOULD prefer locally cached images and MUST support `--local` for development against local compose files.

10. **Env var ↔ CLI flag parity.** Every `EVAL_*` environment variable documented in the README or used by any `oci://` compose artifact MUST have a matching `--kebab-case` flag derived by stripping `EVAL_` and lowercasing, a positional shortcut MUST NOT replace its flag form, a set CLI flag MUST override the env var, and `eval-containers run` MUST translate every flag into its `EVAL_*` env var and shell out to the standard command for the selected `--mode`.

### Commands

11. **Build.** `eval-containers build agent|bench|model|eval` MUST map each target to a single `docker buildx bake <target>` invocation, per-task variants (`--task-id`) MUST fall through to a single `docker build`, in-cluster builds MUST use the same `docker buildx bake` against an in-cluster builder exposed as `build --builder <name>` implying `--push`, and the CLI MUST NOT re-derive the build graph for any platform.

12. **Run.** `eval-containers run {benchmark} --agent {name} --task-id {id}` MUST map to the standard command for the chosen `--mode` (`docker compose up`, `docker run`, or `helm template … \| kubectl apply -f -`), MUST accept both the container-tag and internal-version axes plus `--model`, `--timeout`, and `--local`, and MUST supply platform-specific `job`-mode settings as a composable Helm values file via `--overlay <file>` rather than per-platform CLI code.

13. **Report.** `eval-containers report ./output/` MUST walk the output directory, read `result.json` files, aggregate them, and support `--format csv|json`.

14. **List.** `eval-containers list benchmarks|agents|models` MUST read Docker image labels, with no separate database or index.

15. **Push.** `eval-containers push agent|bench|model|eval` MUST map to `docker push`.

## References

- [Process](../RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added principle 10: env var / CLI flag parity — every `EVAL_*` env var MUST be exposable as a `--kebab-case` flag; CLI flag overrides env var. Updated `eval-containers run` (principle 12) to list the standard version/timeout flags. Renumbered commands (11–15). |
| 2026-04-14 | Updated `eval-containers run` (principle 12) to enumerate both axes of versioning: container tags (`--benchmark-tag`, `--agent-tag`, `--model-tag`) and internal upstream versions (`--benchmark-version`, `--agent-version`, `--litellm-version`). |
| 2026-04-14 | Tightened principle 10 (parity): every `EVAL_*` env var used anywhere in the README or in a published compose artifact MUST have a matching `--kebab-case` flag with no exceptions. Positional shortcuts are allowed but MUST NOT replace the flag form. `eval-containers run`'s job is to translate every flag to its env var and shell out to the exact docker compose command in the README. |
| 2026-05-31 | Restated the CLI's purpose (Abstract): it is a *reminder of the simplest standard command* for each task — every command MUST be reducible to, and able to print, a plain `docker`/`kubectl`/`oc` invocation; anything not expressible that way does not belong in the CLI. Principle 2 reinforced: the CLI exists to discover the command, never to hide it. |
| 2026-05-31 | Generalized the tool surface from Docker-only to the set actually in use: added the **Underlying tools** subsection (docker `build`/`buildx bake`/`compose`/`run`/`push`, `kubectl`+Kustomize, `oc`) and rewrote principles 1–6, 8, 10 to reference it. Principle 2 now requires `--dry-run` to print the underlying commands; principle 3 forbids CLI-resident dependency-ordering — a build/deploy graph MUST be data the tool executes (bake file run by buildx, Kustomize overlay run by `kubectl`/`oc`), linking top-level principle 15. |
| 2026-05-31 | Principle 11 (Build): `docker build` → `docker buildx bake` (post-bake-migration); in-cluster builds are the *same* `docker buildx bake` against a `--driver kubernetes` builder — not a re-derived per-platform build graph. |
| 2026-05-31 | Principle 12 (Run): documented all three modes (compose → `docker compose up`, container → `docker run`, job → `kubectl apply -k`); platform-specific `job` settings are composable Kustomize overlays, not per-platform CLI code. |
| 2026-05-31 | Principle 11 (Build): exposed in-cluster builds as `build --builder <name>` — a passthrough of buildx's `--builder` that implies `--push`; a missing builder fails with the `docker buildx create --driver kubernetes` command to run. |
| 2026-05-31 | Principle 2 refined: scoped "maps to a tool command" to container/cluster-driving commands and carved out `report`/`gen-bake` as local file utilities; scoped the `--dry-run` requirement to state-changing/outward commands (`build`, `run`, `push`, `prune`). |
| 2026-06-01 | `run --mode job` moved from synthesizing a Kustomize overlay to `helm template benchmarks/_chart -f benchmarks/<x>/values.yaml --set … \| kubectl apply -f -`. `--overlay` now takes a Helm values file (e.g. `deploy/values-openshift.yaml`), not a Kustomize component directory. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
