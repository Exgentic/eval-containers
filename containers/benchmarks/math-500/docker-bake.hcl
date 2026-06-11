target "benchmark-math-500" {
  context = "containers/benchmarks/math-500"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/math-500:${TAG}"]
}
