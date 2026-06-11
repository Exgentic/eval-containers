target "benchmark-tau-bench" {
  context = "containers/benchmarks/tau-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/tau-bench:${TAG}"]
}
