target "benchmark-mmmu" {
  context = "containers/benchmarks/mmmu"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/mmmu:${TAG}"]
}
