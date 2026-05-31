target "benchmark-global-mmlu" {
  context = "benchmarks/global-mmlu"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/global-mmlu:latest"]
}
