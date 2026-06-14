target "benchmark-scicode" {
  context = "containers/benchmarks/scicode"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/scicode:${TAG}"]
}
