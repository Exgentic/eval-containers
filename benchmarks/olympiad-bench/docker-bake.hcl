variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-olympiad-bench" {
  context = "benchmarks/olympiad-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/olympiad-bench:latest"]
}
