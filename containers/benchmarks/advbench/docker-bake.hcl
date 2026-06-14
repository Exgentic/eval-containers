target "benchmark-advbench" {
  context = "containers/benchmarks/advbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/advbench:${TAG}"]
}
