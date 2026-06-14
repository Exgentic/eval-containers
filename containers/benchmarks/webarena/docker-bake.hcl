target "benchmark-webarena" {
  context = "containers/benchmarks/webarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/webarena:${TAG}"]
}
