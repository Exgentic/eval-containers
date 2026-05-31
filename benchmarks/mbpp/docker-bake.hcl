target "benchmark-mbpp" {
  context = "benchmarks/mbpp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mbpp:latest"]
}
