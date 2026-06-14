target "benchmark-arena-hard" {
  context = "containers/benchmarks/arena-hard"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/arena-hard:${TAG}"]
}
