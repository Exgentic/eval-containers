variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-cybench" {
  context = "benchmarks/cybench"
  contexts = {
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/cybench:latest"]
}
