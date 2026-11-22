target "benchmark-cvdp-agentic" {
  context = "containers/benchmarks/cvdp-agentic"
  contexts = {
    "${REGISTRY}/core/cvdp-sim" = "target:cvdp-sim"
  }
  tags = ["${REGISTRY}/benchmarks/cvdp-agentic:${TAG}"]
}
