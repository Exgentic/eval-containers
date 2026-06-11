target "benchmark-theoremqa" {
  context = "containers/benchmarks/theoremqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/theoremqa:${TAG}"]
}
