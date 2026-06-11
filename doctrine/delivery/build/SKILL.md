---
name: build
description: >-
  Build the Eval Containers image fleet — or any single artifact or eval
  combination — with `docker buildx bake`, and write the per-artifact
  `docker-bake.hcl` files the build graph requires. Use when building an
  image locally, composing a benchmark × agent × model eval, or adding a
  bake file for a new artifact. For cutting and pushing a tagged release
  of the whole fleet (with the readiness gate), use the `release` skill,
  which wraps this one.
---

# Build images with Docker Bake

The fleet's build graph is data: every artifact under `core/`,
`agents/`, `benchmarks/`, `models/`, and `gateways/` ships a
`docker-bake.hcl` next to its `Dockerfile` declaring its build
dependencies (`doctrine/RULES.md:15`). `docker buildx bake` is the one
canonical build invocation — used identically by humans, the CLI, tests,
and out-of-process consumers (OC in-cluster builds, bakah). This skill
covers building with it and authoring the bake files it reads.

Serves: `doctrine/RULES.md:15` (build graph is data) and its sub-rules,
`doctrine/RULES.md:11` (reuse over repetition — fleet-wide variables
declared once), and `doctrine/RULES.md:8` (clean code — bake files stay
minimal).

The convention for *bake itself* is below; for bake's own semantics see
the [Docker Bake documentation](https://docs.docker.com/build/bake/).

## Building

1. **`REGISTRY` and `TAG` are fleet-wide, declared once at the root.**
   `./docker-bake.hcl` declares `REGISTRY` (default
   `ghcr.io/exgentic`) and `TAG` (default `latest`). Per-artifact
   files reference `${REGISTRY}/...:${TAG}` and never redeclare them
   (`doctrine/RULES.md:15`, sub-rule b — reuse over repetition). Invoking
   `docker buildx bake` from the repo root picks up the root file via
   auto-discovery; wrappers include it explicitly. Why: one place to
   point the whole fleet at a different registry or tag.

2. **Build a single target by name.** Target names are
   `<category>-<name>` (`agent-openhands`, `benchmark-aime`,
   `model-gpt-5_4--bifrost`); leaf `core/` images use their bare
   directory name (`agent-base-python`, `benchmark-base-hf`)
   (`doctrine/RULES.md:15`, sub-rule a):

   ```bash
   docker buildx bake benchmark-aime          # build one artifact
   docker buildx bake benchmark-aime --load   # ...and load into the daemon
   TAG=v1.2.0 docker buildx bake agent-openhands --push
   ```

3. **Compose an eval (benchmark × agent × model).** The combination
   Dockerfile takes any benchmark + agent + model + otel + runtime
   bundle and produces an eval image. Its bake target is parameterized;
   concrete combos are composed at call time by chaining every
   dependency's `-f` file and overriding the image-ref args. Bake merges
   all `-f` files into one graph. Example —
   `aime × openhands × gpt-5.4--bifrost`:

   ```bash
   docker buildx bake \
     -f core/agent-base-python/docker-bake.hcl \
     -f core/benchmark-base-hf/docker-bake.hcl \
     -f core/test-exact-match/docker-bake.hcl \
     -f core/otel/docker-bake.hcl \
     -f core/runtime-bundle/docker-bake.hcl \
     -f gateways/bifrost/docker-bake.hcl \
     -f models/gpt-5.4--bifrost/docker-bake.hcl \
     -f benchmarks/aime/docker-bake.hcl \
     -f agents/openhands/docker-bake.hcl \
     -f core/combination.docker-bake.hcl \
     --set "eval.args.BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/aime:latest" \
     --set "eval.args.AGENT_IMAGE=ghcr.io/exgentic/agents/openhands:latest" \
     --set "eval.args.MODEL_IMAGE=ghcr.io/exgentic/models/gpt-5.4--bifrost:latest" \
     --load eval
   ```

   Why so many `-f` files: each artifact owns exactly one target file
   (sub-rule a), so a combo lists every dependency explicitly. Listing
   ten files by hand is tedious in practice — prefer the wrapper in the
   next step.

4. **Prefer the CLI wrapper over hand-chaining `-f` files.** The
   framework's CLI (`eval-containers build eval <bench> --agent <agent>`)
   and the OC build script wrap the composition in step 3, always
   including the root `./docker-bake.hcl`. Hand-chain `-f` files only
   when debugging the graph directly. Why: the wrapper is the
   maintained composition; hand-listing drifts as dependencies change.

5. **For the whole fleet at a release tag, use the `release` skill.**
   Building and pushing every image with the readiness gate is the
   `release` skill (`doctrine/release/SKILL.md`), which wraps this one.
   The single-artifact and combo flows above are the local dev loop.

6. **Podman note.** Bake requires `buildx`, which podman's docker-compat
   shim doesn't ship. To run a fleet build locally against podman,
   install the real Docker CLI alongside podman, but in practice let CI
   do fleet builds — humans build one thing at a time:

   ```bash
   brew install docker-buildx
   export DOCKER_HOST=unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')
   docker buildx bake benchmark-aime --print
   ```

## Authoring a bake file

7. **Scaffold new files with the generator, don't hand-write.** For a
   new artifact:

   ```bash
   eval-containers gen-bake agents/foo
   # wrote agents/foo/docker-bake.hcl
   ```

   The generator parses the artifact's `Dockerfile` (`FROM` +
   `COPY --from=`), emits the bake file in the canonical shape, and
   exits. Re-running on an existing file requires `--force`. Output
   passes the lint by construction. Why: hand-copying drifts; the
   generator derives the contexts from the real Dockerfile.

8. **Follow the minimal template — one target, the full surface.** A
   bake file owns its artifact's single target and nothing else
   (`doctrine/RULES.md:15`, sub-rule a). The entire surface:

   ```hcl
   target "<category>-<name>" {
     context  = "<category>/<name>"
     contexts = {
       "${REGISTRY}/<dep-category>/<dep-name>" = "target:<dep-target-name>"
       # ... one entry per in-repo FROM / COPY --from=
     }
     args = { HF_TOKEN = HF_TOKEN }   # only if the Dockerfile takes it
     tags = ["${REGISTRY}/<category>/<name>:${TAG}"]
   }
   ```

   A leaf core image with no in-repo deps comes in at 4 lines (no
   `contexts` block). Per-artifact templates by type:

   - **Leaf core** (`core/agent-base-python`, `core/otel`,
     `core/entrypoint`) — `FROM` only upstream registries, so no
     `contexts`:

     ```hcl
     target "agent-base-python" {
       context = "core/agent-base-python"
       tags    = ["${REGISTRY}/core/agent-base-python:${TAG}"]
     }
     ```

   - **Core with one in-repo dep** (`core/benchmark-base-hf`):

     ```hcl
     variable "HF_TOKEN" { default = "" }

     target "benchmark-base-hf" {
       context  = "core/benchmark-base-hf"
       contexts = { "${REGISTRY}/core/entrypoint:${TAG}" = "target:entrypoint" }
       args     = { HF_TOKEN = HF_TOKEN }
       tags     = ["${REGISTRY}/core/benchmark-base-hf:${TAG}"]
     }
     ```

   - **Agent** (`agents/openhands`) — extends its runtime base
     (`doctrine/RULES.md:11`, sub-rule a). Per-agent upstream versions
     live in the Dockerfile as `ARG AGENT_VERSION=x.y.z`
     (`doctrine/RULES.md:9` — internal version axis), driving the install
     + label, NOT the bake tag; the tag is the framework's container
     version, set fleet-wide via `${TAG}`:

     ```hcl
     target "agent-openhands" {
       context  = "agents/openhands"
       contexts = { "${REGISTRY}/core/agent-base-python" = "target:agent-base-python" }
       tags     = ["${REGISTRY}/agents/openhands:${TAG}"]
     }
     ```

   - **Benchmark** (`benchmarks/aime`) — extends a benchmark base and a
     grader (`doctrine/RULES.md:11`, sub-rule b):

     ```hcl
     variable "HF_TOKEN" { default = "" }

     target "benchmark-aime" {
       context  = "benchmarks/aime"
       contexts = {
         "${REGISTRY}/core/benchmark-base-hf:${TAG}" = "target:benchmark-base-hf"
         "${REGISTRY}/core/test-exact-match:${TAG}"  = "target:test-exact-match"
       }
       args = { HF_TOKEN = HF_TOKEN }
       tags = ["${REGISTRY}/benchmarks/aime:${TAG}"]
     }
     ```

   - **Gateway** (`gateways/bifrost`) and **model**
     (`models/gpt-5.4--bifrost`) follow the same shape; a model `FROM`s
     its gateway:

     ```hcl
     target "model-gpt-5_4--bifrost" {
       context  = "models/gpt-5.4--bifrost"
       contexts = { "${REGISTRY}/gateways/bifrost:${TAG}" = "target:gateway-bifrost" }
       tags     = ["${REGISTRY}/models/gpt-5.4--bifrost:${TAG}"]
     }
     ```

   - **Combination** (`core/combination.docker-bake.hcl`) — the only
     parameterized template; image refs come in as
     orchestration-composed variables (`doctrine/RULES.md:15`,
     sub-rule h):

     ```hcl
     variable "EVAL_BENCHMARK"     {}   # required
     variable "EVAL_AGENT"         {}   # required
     variable "EVAL_AGENT_VERSION" { default = "" }
     variable "BENCHMARK_IMAGE"    {}
     variable "AGENT_IMAGE"        {}
     variable "MODEL_IMAGE"        {}
     variable "OTEL_IMAGE"         { default = "${REGISTRY}/core/otel:${TAG}" }
     variable "RUNTIME_BUNDLE_IMAGE" { default = "${REGISTRY}/core/runtime-bundle:${TAG}" }

     target "eval" {
       context    = "."
       dockerfile = "core/combination.Dockerfile"
       args = {
         BENCHMARK_IMAGE      = BENCHMARK_IMAGE
         AGENT_IMAGE          = AGENT_IMAGE
         AGENT_VERSION        = EVAL_AGENT_VERSION
         MODEL_IMAGE          = MODEL_IMAGE
         OTEL_IMAGE           = OTEL_IMAGE
         RUNTIME_BUNDLE_IMAGE = RUNTIME_BUNDLE_IMAGE
       }
       tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}:${TAG}"]
     }
     ```

9. **Map every in-repo `FROM` / `COPY --from=` into `contexts`.** Each
   in-repo dependency MUST appear in the target's `contexts`, mapping
   the full image reference (`${REGISTRY}/...:${TAG}`) to the dep's bake
   target (`target:<name>`) (`doctrine/RULES.md:15`, sub-rule d). This
   is what makes the graph explicit and consumable by every build tool.
   Why: a `FROM` that isn't in `contexts` is invisible to the build
   graph — the lint catches the mismatch.

10. **Build-time secrets go through `args` with empty defaults.** Pass
    `args = { HF_TOKEN = HF_TOKEN }` and declare
    `variable "HF_TOKEN" { default = "" }` only when the Dockerfile
    actually takes that `ARG` (`doctrine/RULES.md:15`, sub-rules e, g).
    Artifacts that don't need a secret pay no ceremony to declare it
    absent. Why: forwarding an unused arg "just in case" fails the lint.

11. **Keep variable hygiene strict.** A `variable` MUST exist for
    exactly one of: a per-build override (`REGISTRY`, `TAG` — at root
    only), a build-time secret (`HF_TOKEN`), or an
    orchestration-composed reference (`BENCHMARK_IMAGE`, `AGENT_IMAGE`,
    `MODEL_IMAGE` in the combo template) (`doctrine/RULES.md:15`,
    sub-rule h). Values that ARE the artifact — target name, context
    directory, the `FROM` graph shape, the `:${TAG}` suffix itself —
    MUST be hardcoded. Artifact-scoped variables stay in the artifact's
    own file; fleet-wide ones live at root. Every `variable` MUST be
    referenced — dead declarations fail the lint. Why: a `variable` is
    configuration, not a hiding place for artifact identity.

## Conciseness conventions (the lint trips on each)

These are normative per `doctrine/RULES.md:15`, sub-rule g (Minimal),
and `doctrine/RULES.md:8` (clean code). Mechanical enforcement —
every artifact has a valid bake file whose `contexts` match its
Dockerfile's `FROM` lines — lives in the build-test catalog
(`tests/build/RULES.md`).

12. **One target per file.** A bake file owns its artifact's target and
    nothing else. No grouping targets across artifacts; no multi-target
    files. If you want two targets in one file, you have two artifacts —
    put each in its own directory.

13. **No `inherits` chains.** Each target's properties are explicit so
    readers don't trace inheritance graphs across files.

14. **No `group "default"`.** Always name the target you want.

15. **Variables go at the top, scoped tight.** `variable` declarations
    come before the `target`. Per-artifact files declare only what's
    scoped to them (secrets); `REGISTRY` and `TAG` stay at root.

16. **No comments restating the rule.** This skill and `doctrine/RULES.md`
    are the rule; a per-artifact bake file's existence is its citation.
    No header explaining what bake is.

17. **No `dockerfile-inline`.** Always reference the real `Dockerfile`
    in the artifact's directory; inline Dockerfiles obscure the build.

18. **`args` only for declared build-args.** If the Dockerfile takes no
    such `ARG`, the bake file passes no `args` value for it.

## What goes elsewhere, not in bake files

- **Build orchestration** (which artifacts to build for a given combo):
  `src/build.rs` and the CLI.
- **Compose runtime topology**: each benchmark's `compose.yaml`.
- **K8s runtime topology**: the shared Helm chart `benchmarks/_chart`, selected with `--set benchmark=<x>` + an optional `presets/<x>.yaml` for bespoke topology.
- **Per-task variant builds** (swe-bench's 1000+ tasks): the CLI's
  `--task-id` flow; not enumerated in bake.
- **In-cluster OC builds**: a translator that reads bake files and emits
  `oc start-build` calls; lives under `oc/`.

## References

- `doctrine/RULES.md:15` — Build graph is data (and its sub-rules a–h).
- [Docker Bake documentation](https://docs.docker.com/build/bake/)
- [Bakah (daemonless bake for podman/buildah)](https://github.com/emersion/bakah)
- The `release` skill (`doctrine/release/SKILL.md`) — the fleet-wide
  tagged push that wraps this skill.
- `tests/build/RULES.md` — the mechanical bake-alignment
  lint and the build sweep.
