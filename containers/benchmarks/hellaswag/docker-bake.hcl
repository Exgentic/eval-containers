target "benchmark-hellaswag" {
  context = "containers/benchmarks/hellaswag"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/hellaswag:${TAG}"]
}
