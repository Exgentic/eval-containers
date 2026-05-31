variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-piqa" {
  context = "benchmarks/piqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/piqa:latest"]
}
