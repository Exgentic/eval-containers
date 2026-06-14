target "benchmark-commonsenseqa" {
  context = "containers/benchmarks/commonsenseqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/commonsenseqa:${TAG}"]
}
