variable "EVAL_BENCHMARK"     {}
variable "EVAL_AGENT"         {}
variable "EVAL_AGENT_VERSION" { default = "" }
variable "BENCHMARK_IMAGE"    {}
variable "AGENT_IMAGE"        {}
variable "MODEL_IMAGE"        { default = "${REGISTRY}/models/bifrost:${TAG}" }
variable "OTEL_IMAGE"         { default = "${REGISTRY}/core/otel:${TAG}" }
variable "GOSU_IMAGE"            { default = "${REGISTRY}/core/gosu:${TAG}" }
variable "PROCESS_COMPOSE_IMAGE" { default = "${REGISTRY}/core/process-compose:${TAG}" }

# Lean eval base (evals/<b>--<a>:latest): benchmark + agent + grader + the
# framework launcher (gosu/run/run-agent/write-result). No gateway, otelcol, or
# process-compose — that is the standalone bundle below. This is what
# `--mode compose`/`job`/k8s run, with the gateway + otelcol as sidecars.
target "eval" {
  context    = "containers/core"
  dockerfile = "combination.Dockerfile"
  # Build gosu in-graph so eval needs no pre-published core/gosu (offline).
  contexts = {
    "${REGISTRY}/core/gosu" = "target:gosu"
  }
  args = {
    BENCHMARK_IMAGE = BENCHMARK_IMAGE
    AGENT_IMAGE     = AGENT_IMAGE
    AGENT_VERSION   = EVAL_AGENT_VERSION
    GOSU_IMAGE      = GOSU_IMAGE
  }
  tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}:${TAG}"]
}

# Local-build variant: wires bench+agent in-graph via named contexts so eval
# builds them as graph dependencies instead of pulling from the registry.
# Used by `build eval --no-pull` on base (non-task) builds to avoid registry
# manifest checks that fail on arm64 Mac (docker-container driver isolation
# means --load'd images are not visible in the BuildKit content store).
target "eval-local" {
  inherits = ["eval"]
  contexts = {
    "${BENCHMARK_IMAGE}" = "target:benchmark-${EVAL_BENCHMARK}"
    "${AGENT_IMAGE}"     = "target:agent-${EVAL_AGENT}"
  }
}

# Single-container standalone bundle (evals/<b>--<a>-standalone:<version>): the
# lean base + the in-process gateway, otelcol, process-compose, and the full
# pipeline. The laptop / `--mode container` artifact. The variant is a NAME
# suffix, never the tag — the `:tag` is the release version (top-level RULES.md
# principle 9), so the bundle shares the lean base's tag and differs by name.
# The `eval-base` named context wires standalone.Dockerfile's `FROM eval-base`
# to the lean `eval` target, so `bake eval-standalone` builds the lean base
# in-graph and layers onto its output directly — a real build-graph node
# (src/RULES.md P11), no registry/cache round-trip (a literal context name binds
# where an ARG-based FROM does not).
target "eval-standalone" {
  context    = "containers/core"
  dockerfile = "standalone.Dockerfile"
  contexts = {
    "eval-base"                        = "target:eval"
    "${REGISTRY}/core/process-compose" = "target:process-compose"
  }
  args = {
    MODEL_IMAGE           = MODEL_IMAGE
    OTEL_IMAGE            = OTEL_IMAGE
    PROCESS_COMPOSE_IMAGE = PROCESS_COMPOSE_IMAGE
  }
  tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}-standalone:${TAG}"]
}
