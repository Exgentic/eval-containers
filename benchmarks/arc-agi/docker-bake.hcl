target "benchmark-arc-agi" {
  context = "benchmarks/arc-agi"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/arc-agi:latest"]
}
