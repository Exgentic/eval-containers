variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-agentbench" {
  context = "benchmarks/agentbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github:latest" = "target:benchmark-base-github"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/agentbench:latest"]
}
