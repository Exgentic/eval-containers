target "benchmark-mmmu" {
  context = "containers/benchmarks/mmmu"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/mmmu:${TAG}"]
}
