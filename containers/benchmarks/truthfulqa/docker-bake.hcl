target "benchmark-truthfulqa" {
  context = "containers/benchmarks/truthfulqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  tags = ["${REGISTRY}/benchmarks/truthfulqa:${TAG}"]
}
