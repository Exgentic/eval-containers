variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-agentcompany" {
  context = "benchmarks/agentcompany"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/agentcompany:latest"]
}
