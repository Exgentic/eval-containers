variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-bbh" {
  context = "benchmarks/bbh"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github:latest" = "target:benchmark-base-github"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/bbh:latest"]
}
