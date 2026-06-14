target "benchmark-coderefine" {
  context = "containers/benchmarks/coderefine"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/coderefine:${TAG}"]
}
