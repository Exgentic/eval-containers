target "benchmark-mbpp" {
  context = "containers/benchmarks/mbpp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mbpp:${TAG}"]
}
