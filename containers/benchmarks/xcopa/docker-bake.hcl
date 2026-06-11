target "benchmark-xcopa" {
  context = "containers/benchmarks/xcopa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/xcopa:${TAG}"]
}
