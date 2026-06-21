target "benchmark-niah" {
  context = "containers/benchmarks/niah"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/niah:${TAG}"]
}
