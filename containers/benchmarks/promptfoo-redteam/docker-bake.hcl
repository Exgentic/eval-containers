target "benchmark-promptfoo-redteam" {
  context = "containers/benchmarks/promptfoo-redteam"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/promptfoo-redteam:${TAG}"]
}
