variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-xnli" {
  context = "benchmarks/xnli"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/xnli:latest"]
}
