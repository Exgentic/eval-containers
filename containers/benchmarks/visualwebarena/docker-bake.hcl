target "benchmark-visualwebarena" {
  context = "containers/benchmarks/visualwebarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/visualwebarena:${TAG}"]
}
