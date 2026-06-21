target "benchmark-piqa" {
  context = "containers/benchmarks/piqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  tags = ["${REGISTRY}/benchmarks/piqa:${TAG}"]
}
