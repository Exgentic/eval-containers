variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-gsm8k" {
  context = "benchmarks/gsm8k"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/gsm8k:latest"]
}
