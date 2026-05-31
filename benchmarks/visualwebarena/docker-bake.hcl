target "benchmark-visualwebarena" {
  context = "benchmarks/visualwebarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/visualwebarena:${TAG}"]
}
