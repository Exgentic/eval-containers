variable "REGISTRY"             { default = "quay.io/eval-containers" }
variable "EVAL_BENCHMARK"       {}
variable "EVAL_AGENT"           {}
variable "EVAL_AGENT_VERSION"   { default = "latest" }
variable "BENCHMARK_IMAGE"      {}
variable "AGENT_IMAGE"          {}
variable "MODEL_IMAGE"          {}
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
