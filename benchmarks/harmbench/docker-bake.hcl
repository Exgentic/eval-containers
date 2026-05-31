variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-harmbench" {
  context = "benchmarks/harmbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/harmbench:latest"]
}
