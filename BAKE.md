# Bake convention

**Status:** Active
**Date:** May 2026

How to write the bake files required by [`RULES.md`](RULES.md) principle 15.
This is a convention guide, not a tutorial on bake itself — for that, see
[Docker Bake documentation](https://docs.docker.com/build/bake/).

The goal: every `docker-bake.hcl` in the tree is **standalone**, **concise**,
and follows a uniform template so contributors don't reinvent the structure
per artifact.

## The minimal template

Every bake file follows this shape:

```hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }

target "<category>-<name>" {
  context  = "<category>/<name>"
  contexts = {
    "${REGISTRY}/<dep-category>/<dep-name>:<tag>" = "target:<dep-target-name>"
    # ... one entry per in-repo FROM / COPY --from=
  }
  args = { HF_TOKEN = HF_TOKEN }   # only if the Dockerfile takes it
  tags = ["${REGISTRY}/<category>/<name>:<version>"]
}
```

That's the entire surface. Most files come in under 10 lines.

## Per-artifact templates

### Leaf core image (no in-repo deps)

For images like `core/agent-base-python`, `core/otel`, `core/entrypoint`
that `FROM` only upstream registries (Docker Hub, ghcr, etc.) — no contexts
block needed:

```hcl
# core/agent-base-python/docker-bake.hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-base-python" {
  context = "core/agent-base-python"
  tags    = ["${REGISTRY}/core/agent-base-python:latest"]
}
```

### Core image with one in-repo dep

For images like `core/benchmark-base-hf` that `FROM` another in-repo image:

```hcl
# core/benchmark-base-hf/docker-bake.hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-base-hf" {
  context  = "core/benchmark-base-hf"
  contexts = { "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint" }
  args     = { HF_TOKEN = HF_TOKEN }
  tags     = ["${REGISTRY}/core/benchmark-base-hf:latest"]
}
```

### Gateway

```hcl
# gateways/bifrost/docker-bake.hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }

target "bifrost" {
  context = "gateways/bifrost"
  tags    = ["${REGISTRY}/gateways/bifrost:latest"]
}
```

### Agent

Pinned upstream version is a variable so it can be overridden without
editing the file:

```hcl
# agents/openhands/docker-bake.hcl
variable "REGISTRY"          { default = "quay.io/eval-containers" }
variable "OPENHANDS_VERSION" { default = "1.7.0" }

target "agent-openhands" {
  context  = "agents/openhands"
  contexts = { "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python" }
  tags     = ["${REGISTRY}/agents/openhands:${OPENHANDS_VERSION}"]
}
```

### Benchmark

```hcl
# benchmarks/aime/docker-bake.hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-aime" {
  context  = "benchmarks/aime"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest"  = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/aime:latest"]
}
```

### Model

```hcl
# models/gpt-5.4--bifrost/docker-bake.hcl
variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-gpt-5_4--bifrost" {
  context  = "models/gpt-5.4--bifrost"
  contexts = { "${REGISTRY}/gateways/bifrost:latest" = "target:bifrost" }
  tags     = ["${REGISTRY}/models/gpt-5.4--bifrost:latest"]
}
```

### Combination (the eval template)

The combination Dockerfile takes any benchmark + agent + model + otel +
runtime-bundle and produces an eval image. The bake template is
parameterized; concrete combos are composed at call time via `-f` chaining
plus `--set` overrides.

```hcl
# core/combination.docker-bake.hcl
variable "REGISTRY"             { default = "quay.io/eval-containers" }
variable "EVAL_BENCHMARK"       {}   # required
variable "EVAL_AGENT"           {}   # required
variable "EVAL_AGENT_VERSION"   { default = "latest" }
variable "BENCHMARK_IMAGE"      {}   # required image ref
variable "AGENT_IMAGE"          {}   # required image ref
variable "MODEL_IMAGE"          {}   # required image ref
variable "OTEL_IMAGE"           { default = "quay.io/eval-containers/core/otel:latest" }
variable "RUNTIME_BUNDLE_IMAGE" { default = "quay.io/eval-containers/core/runtime-bundle:latest" }

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
  tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}:${EVAL_AGENT_VERSION}"]
}
```

## Composing for an eval

To build `aime × openhands × gpt-5.4--bifrost`:

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
  --set "eval.args.BENCHMARK_IMAGE=quay.io/eval-containers/benchmarks/aime:latest" \
  --set "eval.args.AGENT_IMAGE=quay.io/eval-containers/agents/openhands:1.7.0" \
  --set "eval.args.MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--bifrost:latest" \
  --load   eval
```

Bake merges all the `-f` files into one graph. Listing 10 files is
tedious in practice — the framework's CLI (`eval-containers build eval
<bench> --agent <agent>`) and the OC build script wrap this composition.
The bake files themselves stay standalone.

## Conventions for conciseness

1. **One target per file.** A bake file owns its artifact's target and
   nothing else. No grouping targets across artifacts.

2. **No `inherits` chains.** Bake supports target inheritance; we don't
   use it. Each target's properties are explicit so readers don't trace
   inheritance graphs across files.

3. **No `group "default"`.** Defaults are confusing in the per-artifact
   pattern. Always name the target you want.

4. **Variables go at the top.** `variable` declarations come before the
   `target`. Order: `REGISTRY`, then secrets (`HF_TOKEN`), then versions.

5. **No comments restating the rule.** This file is the rule. Per-artifact
   bake files don't need a header explaining what bake is or citing
   principle 15 — the file's existence is the citation.

6. **No `dockerfile-inline`.** Always reference the real `Dockerfile`
   in the artifact's directory. Inline Dockerfiles obscure the build.

7. **Keep `args` to declared build-args only.** If the Dockerfile doesn't
   take an `ARG`, the bake file doesn't pass an `args` value for it. No
   forwarding "just in case."

8. **No multi-target files.** If you find yourself wanting two targets
   in one file, you have two artifacts. Put each in its own directory.

## What goes elsewhere

- **Build orchestration** (which artifacts to build for a given
  benchmark/agent/model combo): `src/build.rs` and the CLI.
- **Compose runtime topology**: each benchmark's `compose.yaml`.
- **K8s runtime topology**: each benchmark's `kustomization.yaml`.
- **Per-task variant builds** (swe-bench 1000+ tasks): the CLI's
  `--task-id` flow; not enumerated in bake.
- **In-cluster OC builds**: a translator that reads bake files and emits
  `oc start-build` calls; lives under `oc/`.

## References

- [RULES.md](RULES.md) — principle 15 (Build graph is data)
- [Docker Bake documentation](https://docs.docker.com/build/bake/)
- [Bakah (daemonless bake for podman/buildah)](https://github.com/emersion/bakah)

## Changelog

| Date | Change |
|------|--------|
| 2026-05-31 | Initial version. |
