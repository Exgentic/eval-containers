target "benchmark-webarena" {
  context = "benchmarks/webarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/webarena:${TAG}"]
}
