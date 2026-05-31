variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-ai2d" {
  context = "benchmarks/ai2d"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/ai2d:latest"]
}
