target "benchmark-minif2f" {
  context = "containers/benchmarks/minif2f"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  tags = ["${REGISTRY}/benchmarks/minif2f:${TAG}"]
}
