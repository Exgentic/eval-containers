target "benchmark-arena-hard" {
  context = "benchmarks/arena-hard"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/arena-hard:latest"]
}
