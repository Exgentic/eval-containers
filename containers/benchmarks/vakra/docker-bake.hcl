target "benchmark-vakra" {
  context = "containers/benchmarks/vakra"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/vakra:${TAG}"]
}
