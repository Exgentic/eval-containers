# CLI

**Status:** Active
**Date:** April 2026

## Abstract

The `eval-containers` CLI is a **reminder of the simplest standard way to run an eval** — not a layer over it. Every command is a mnemonic for a plain `docker` / `kubectl` / `oc` invocation you could type yourself; running `eval-containers <X>` MUST be reducible to, and able to print, that exact command. It is a thin Rust wrapper around the standard container, Kubernetes, and OpenShift tools — Docker (including `docker buildx bake` and `docker compose`), `kubectl` with Kustomize, and `oc` — and it exists to save keystrokes, not to add abstractions. If a task cannot be expressed as a standard-tool command, it does not belong in the CLI. This document defines the design principles for the CLI.

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
| `oc` | applying manifests on OpenShift (`helm template … \| oc apply -f -`); single-artifact in-cluster builds via `BuildConfig` (`oc start-build`) for `build --builder oc`; the `kubectl` superset for OpenShift login and registry routing |

Each tool MUST be a standard release the user can install and invoke themselves — no forks, no wrappers. Adding a tool to this list is a rule change and MUST be recorded in the changelog; the CLI MUST NOT reach for any tool outside it.

### Philosophy

1. **Optional.** The CLI MUST be optional. Everything it does MUST be reproducible by running the underlying tools (§Underlying tools) by hand. The CLI is a shortcut, not a dependency.

2. **Transparent.** Every command that drives containers or clusters MUST map to one or more underlying-tool commands (§Underlying tools) — never to behavior with no hand-runnable equivalent. (`report` and `gen-bake` are local utilities that only read or write the repo's own files and outputs; they map to no container tool, but MUST stay equally transparent and reproducible by hand — `find`/`jq`, or a text editor.) The CLI SHOULD print the underlying command(s) it runs; commands that change state or reach outward (`build`, `run`, `push`, `prune`) MUST support `--dry-run`, which prints those commands without executing them. The user MUST be able to reproduce the result by running those commands themselves — running the CLI is a way to discover the command it stands for, never to hide it.

3. **No magic.** The CLI MUST NOT introduce abstractions beyond what the underlying tools provide. No custom runtimes, no hidden state, no daemons, no orchestration that lives in the CLI. Where a build or deployment spans a *dependency* graph of artifacts, that graph MUST be expressed as data the tool executes — a `docker-bake.hcl` (top-level RULES.md principle 15) run by buildx, or a Kustomize overlay run by `kubectl`/`oc` — never as ordering logic re-derived inside the CLI. If the tools can't do it, Eval Containers doesn't promise it.

### Implementation

4. **Rust.** The CLI is written in Rust. It MUST be a single static binary with no runtime dependencies beyond the tools it shells out to: Docker for the default surfaces, and `kubectl`/`oc` only for the Kubernetes and OpenShift surfaces.

5. **Shell out.** The CLI MUST invoke the underlying tools (§Underlying tools) as subprocesses. It MUST NOT reimplement their functionality. It MUST NOT use their client libraries (e.g. the Docker or Kubernetes Go clients) when a shell command suffices.

6. **Simplest implementation.** Each command SHOULD be the shortest path to calling the right underlying tool with the right arguments. Prefer string formatting over abstractions.

### Behavior

7. **Auto-build.** When running an evaluation, the CLI SHOULD build any missing images (eval, model, agent) before starting. It MUST NOT rebuild images that already exist locally.

8. **Registry-aware.** The CLI MUST use `EVAL_REGISTRY` for all image paths. The same commands MUST work against any OCI-compliant registry, including `localhost:5000` and the OpenShift internal registry (`image-registry.openshift-image-registry.svc:5000`).

9. **Local-first.** The CLI SHOULD prefer locally cached images. It MUST support `--local` for development against local compose files.

10. **Env var ↔ CLI flag parity.** Every `EVAL_*` environment variable documented in the README or used by any `oci://` compose artifact MUST have a matching `--kebab-case` CLI flag derived by stripping `EVAL_` and lowercasing: `EVAL_BENCHMARK` → `--benchmark`, `EVAL_AGENT_VERSION` → `--agent-version`, `EVAL_TASK_ID` → `--task-id`, `EVAL_TIMEOUT` → `--timeout`, `EVAL_LITELLM_VERSION` → `--litellm-version`, and so on. No exceptions: if it's an env var the user can set, it MUST have a flag form. Positional shortcuts (e.g. `eval-containers run aime` accepting `aime` as the benchmark) are allowed but MUST NOT replace the corresponding `--flag`; both forms MUST work. When both a CLI flag and an env var are set, the CLI flag MUST override the env var. The CLI's sole job in `eval-containers run` is to translate every flag into its corresponding `EVAL_*` env var and shell out to the standard command for the selected `--mode` — the exact `docker compose … up`, `docker run …`, or `helm template … \| kubectl apply -f -` the README documents.

### Commands

11. **Build.** `eval-containers build agent|bench|model|eval` — each MUST map to a single `docker buildx bake <target>` invocation, which executes the artifact's build graph declared in its `docker-bake.hcl` (top-level RULES.md principle 15). Per-task variants (`--task-id`), which sit outside the static bake graph, fall through to a single `docker build`. Building in a cluster is **not** a separate code path: it is the same `docker buildx bake` pointed at an in-cluster builder (`docker buildx create --driver kubernetes`). The CLI exposes this as `build --builder <name>` — a passthrough of buildx's own `--builder` that implies `--push` (a remote builder can't load into local Docker); a missing builder fails with the one-time `docker buildx create` command to run. The reserved value `build --builder oc` selects the **OpenShift `BuildConfig` backend** instead of buildx: it builds a single artifact in-cluster with `oc start-build` (buildah under the platform's `builder` SCC) — the no-admin path where baseline PodSecurity blocks in-cluster BuildKit. It MUST read the artifact's resolved build spec — `context`, `dockerfile`, and the eval combination's base-image args — from `docker buildx bake --print <target>` (the bake file stays the only build-graph artifact, principle 3); it adds only the OpenShift-specific translation: the single-segment imagestream naming (`core/otel` → `core-otel`, `benchmarks/aime` → `aime`) and the `REGISTRY`/`REGISTRY_SUFFIX` build args that resolve the parameterized `${REGISTRY}/...${REGISTRY_SUFFIX}` FROMs to the internal registry (binary builds ignore `oc start-build --build-arg`, so they live in the BuildConfig spec). Either backend builds **one** artifact: the CLI MUST NOT re-derive or order the build graph for any platform; dependency-ordered cold-graph builds are a thin loop over `build` that lives outside the CLI (e.g. `examples/openshift/`). Every build also stamps the OCI provenance label `org.opencontainers.image.source` (the pointer GitHub uses to link a package to its repo — on an Actions push or a one-time UI "Connect repository"; a manual push doesn't auto-link) — fleet-wide via `--set "*.labels…"`, and explicit on the per-task `docker build`; never stored in the per-artifact bake files (top-level principle 15.f).

12. **Run.** `eval-containers run {benchmark} --agent {name} --task-id {id}` — maps to the standard command for the chosen `--mode`: `docker compose up` (compose, the default), `docker run` (container), or `helm template <chart> -f <benchmark values> --set … \| kubectl apply -f -` (job). MUST accept both the container-tag axis (`--benchmark-tag`, `--agent-tag`, `--model-tag`) and the internal-version axis (`--benchmark-version`, `--agent-version`, `--litellm-version`), plus `--model`, `--timeout`, `--local`. Cluster- and platform-specific settings for `job` mode (e.g. the service account an OpenShift cluster requires) MUST be supplied as a composable Helm values file via `--overlay <file>` — not encoded per-platform inside the CLI (principle 3); the reference OpenShift overlay is `deploy/values-openshift.yaml`.

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
| 2026-05-31 | Restated the CLI's purpose (Abstract): it is a *reminder of the simplest standard command* for each task — every command MUST be reducible to, and able to print, a plain `docker`/`kubectl`/`oc` invocation; anything not expressible that way does not belong in the CLI. Principle 2 reinforced: the CLI exists to discover the command, never to hide it. |
| 2026-05-31 | Generalized the tool surface from Docker-only to the set actually in use: added the **Underlying tools** subsection (docker `build`/`buildx bake`/`compose`/`run`/`push`, `kubectl`+Kustomize, `oc`) and rewrote principles 1–6, 8, 10 to reference it. Principle 2 now requires `--dry-run` to print the underlying commands; principle 3 forbids CLI-resident dependency-ordering — a build/deploy graph MUST be data the tool executes (bake file run by buildx, Kustomize overlay run by `kubectl`/`oc`), linking top-level principle 15. |
| 2026-05-31 | Principle 11 (Build): `docker build` → `docker buildx bake` (post-bake-migration); in-cluster builds are the *same* `docker buildx bake` against a `--driver kubernetes` builder — not a re-derived per-platform build graph. |
| 2026-05-31 | Principle 12 (Run): documented all three modes (compose → `docker compose up`, container → `docker run`, job → `kubectl apply -k`); platform-specific `job` settings are composable Kustomize overlays, not per-platform CLI code. |
| 2026-05-31 | Principle 11 (Build): exposed in-cluster builds as `build --builder <name>` — a passthrough of buildx's `--builder` that implies `--push`; a missing builder fails with the `docker buildx create --driver kubernetes` command to run. |
| 2026-05-31 | Principle 2 refined: scoped "maps to a tool command" to container/cluster-driving commands and carved out `report`/`gen-bake` as local file utilities; scoped the `--dry-run` requirement to state-changing/outward commands (`build`, `run`, `push`, `prune`). |
| 2026-06-01 | `run --mode job` moved from synthesizing a Kustomize overlay to `helm template benchmarks/_chart -f benchmarks/<x>/values.yaml --set … \| kubectl apply -f -`. `--overlay` now takes a Helm values file (e.g. `deploy/values-openshift.yaml`), not a Kustomize component directory. |
| 2026-06-03 | `run --mode job` drops the per-benchmark `-f values.yaml`; the benchmark is named via `--set benchmark=<x>` and its bespoke topology (if any) lives in the chart at `presets/<x>.yaml`. The chart is now self-contained (renders with no external file), so it can be packaged/published. |
| 2026-06-03 | Principle 11 (Build) + tools table: added the reserved `build --builder oc` OpenShift `BuildConfig` backend — reads the build spec (context, dockerfile, eval base-image args) from `docker buildx bake --print` (bake stays the source of truth) and translates it to `oc start-build` (buildah), the no-admin in-cluster path where baseline PodSecurity blocks BuildKit. One artifact, no graph ordering in the CLI; the ordered loop lives in `examples/openshift/`. |
| 2026-06-11 | Principle 11 (Build): every build stamps the OCI provenance label `org.opencontainers.image.source` (the pointer GitHub uses to link a package to its repo, via an Actions push or UI connect — not a manual push) — fleet-wide via `--set "*.labels…"`, explicit on per-task `docker build`; not in the per-artifact bake files (15.f). |
