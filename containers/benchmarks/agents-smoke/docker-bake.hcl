target "benchmark-agents-smoke" {
  context = "containers/benchmarks/agents-smoke"
  tags = ["${REGISTRY}/benchmarks/agents-smoke:${TAG}"]
}
