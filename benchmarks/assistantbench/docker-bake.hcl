variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-assistantbench" {
  context = "benchmarks/assistantbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/assistantbench:latest"]
}
