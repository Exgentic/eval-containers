target "benchmark-mbppplus" {
  context = "containers/benchmarks/mbppplus"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mbppplus:${TAG}"]
}
