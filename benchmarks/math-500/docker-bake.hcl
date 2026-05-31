variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-math-500" {
  context = "benchmarks/math-500"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/math-500:latest"]
}
