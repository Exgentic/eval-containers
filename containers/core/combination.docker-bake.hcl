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
  # Pulls its bases (gosu/benchmark/agent) from the registry — what the release
  # builds, after the bases job publishes them. Self-contained: references no
  # cross-file target, so the combos job's `-f docker-bake.hcl -f combination`
  # set is enough. Offline / no-registry builds use eval-local (below).
  args = {
    BENCHMARK_IMAGE = BENCHMARK_IMAGE
    AGENT_IMAGE     = AGENT_IMAGE
    AGENT_VERSION   = EVAL_AGENT_VERSION
    GOSU_IMAGE      = GOSU_IMAGE
  }
  tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}:${TAG}"]
}

# In-graph variant (`build eval --no-pull`): inherits eval and binds each base
# FROM to its in-graph target so bake builds benchmark/agent/gosu as graph deps
# instead of pulling them — for offline/empty-registry builds and the arm64-Mac
# docker-container driver isolation where `--load`'d images aren't visible in the
# BuildKit content store. The CLI builds this with the whole graph loaded
# (artifact_bake_files), so these cross-file targets resolve; the release never
# builds it, which is why eval (above) stays self-contained.
target "eval-local" {
  inherits = ["eval"]
  contexts = {
    "${BENCHMARK_IMAGE}"    = "target:benchmark-${EVAL_BENCHMARK}"
    "${AGENT_IMAGE}"        = "target:agent-${EVAL_AGENT}"
    "${REGISTRY}/core/gosu" = "target:gosu"
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
  # eval-base builds the lean eval in-graph (same file); process-compose is
  # pulled via PROCESS_COMPOSE_IMAGE (published by the bases job), keeping this
  # release target self-contained — no cross-file target reference.
  contexts = {
    "eval-base" = "target:eval"
  }
  args = {
    MODEL_IMAGE           = MODEL_IMAGE
    OTEL_IMAGE            = OTEL_IMAGE
    PROCESS_COMPOSE_IMAGE = PROCESS_COMPOSE_IMAGE
  }
  tags = ["${REGISTRY}/evals/${EVAL_BENCHMARK}--${EVAL_AGENT}-standalone:${TAG}"]
}
