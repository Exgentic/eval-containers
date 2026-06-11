target "benchmark-truthfulqa" {
  context = "containers/benchmarks/truthfulqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/truthfulqa:${TAG}"]
}
