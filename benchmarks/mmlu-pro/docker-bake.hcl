variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mmlu-pro" {
  context = "benchmarks/mmlu-pro"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/mmlu-pro:latest"]
}
