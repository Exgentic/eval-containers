target "benchmark-niah" {
  context = "benchmarks/niah"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/niah:latest"]
}
