target "benchmark-mt-bench" {
  context = "containers/benchmarks/mt-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/mt-bench:${TAG}"]
}
