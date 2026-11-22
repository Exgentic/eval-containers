target "benchmark-cvdp" {
  context = "containers/benchmarks/cvdp"
  contexts = {
    "${REGISTRY}/core/cvdp-sim" = "target:cvdp-sim"
  }
  tags = ["${REGISTRY}/benchmarks/cvdp:${TAG}"]
}
