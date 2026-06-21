target "benchmark-arena-hard" {
  context = "containers/benchmarks/arena-hard"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/arena-hard:${TAG}"]
}
