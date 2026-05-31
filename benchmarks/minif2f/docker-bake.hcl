variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-minif2f" {
  context = "benchmarks/minif2f"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/minif2f:latest"]
}
