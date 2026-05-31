target "benchmark-agentdojo" {
  context = "benchmarks/agentdojo"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/agentdojo:${TAG}"]
}
