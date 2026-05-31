variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-agentdojo" {
  context = "benchmarks/agentdojo"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github:latest" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/agentdojo:latest"]
}
