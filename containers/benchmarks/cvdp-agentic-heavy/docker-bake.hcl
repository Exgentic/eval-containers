target "benchmark-cvdp-agentic-heavy" {
  context = "containers/benchmarks/cvdp-agentic-heavy"
  contexts = {
    "${REGISTRY}/core/cvdp-sim" = "target:cvdp-sim"
  }
  tags = ["${REGISTRY}/benchmarks/cvdp-agentic-heavy:${TAG}"]
}
