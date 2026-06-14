target "benchmark-niah" {
  context = "containers/benchmarks/niah"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/niah:${TAG}"]
}
