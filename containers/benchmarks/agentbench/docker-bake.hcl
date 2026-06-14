target "benchmark-agentbench" {
  context = "containers/benchmarks/agentbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/agentbench:${TAG}"]
}
