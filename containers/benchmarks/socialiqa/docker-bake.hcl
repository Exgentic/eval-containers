target "benchmark-socialiqa" {
  context = "containers/benchmarks/socialiqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  tags = ["${REGISTRY}/benchmarks/socialiqa:${TAG}"]
}
