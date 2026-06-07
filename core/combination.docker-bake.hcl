variable "EVAL_BENCHMARK"     {}
variable "EVAL_AGENT"         {}
variable "EVAL_AGENT_VERSION" { default = "" }
variable "BENCHMARK_IMAGE"    {}
variable "AGENT_IMAGE"        {}
variable "MODEL_IMAGE"        {}
variable "OTEL_IMAGE"         { default = "${REGISTRY}/core/otel:${TAG}" }
variable "RUNTIME_BUNDLE_IMAGE" { default = "${REGISTRY}/core/runtime-bundle:${TAG}" }

target "eval" {
  context    = "core"
  dockerfile = "combination.Dockerfile"
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
