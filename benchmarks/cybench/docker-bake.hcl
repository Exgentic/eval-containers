variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-cybench" {
  context = "benchmarks/cybench"
  contexts = {
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/cybench:latest"]
}
