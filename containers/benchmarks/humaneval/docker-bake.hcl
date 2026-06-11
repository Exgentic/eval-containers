target "benchmark-humaneval" {
  context = "containers/benchmarks/humaneval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/humaneval:${TAG}"]
}
