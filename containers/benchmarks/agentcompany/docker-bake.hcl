target "benchmark-agentcompany" {
  context = "containers/benchmarks/agentcompany"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/agentcompany:${TAG}"]
}
