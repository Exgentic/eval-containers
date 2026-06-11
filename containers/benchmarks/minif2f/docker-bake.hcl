target "benchmark-minif2f" {
  context = "containers/benchmarks/minif2f"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/minif2f:${TAG}"]
}
