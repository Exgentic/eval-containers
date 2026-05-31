target "benchmark-olympiad-bench" {
  context = "benchmarks/olympiad-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/olympiad-bench:latest"]
}
