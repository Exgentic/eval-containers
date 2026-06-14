target "benchmark-legalbench" {
  context = "containers/benchmarks/legalbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/legalbench:${TAG}"]
}
