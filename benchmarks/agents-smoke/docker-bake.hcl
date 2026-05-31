variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-agents-smoke" {
  context = "benchmarks/agents-smoke"
  tags = ["${REGISTRY}/benchmarks/agents-smoke:latest"]
}
