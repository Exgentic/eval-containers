variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mt-bench" {
  context = "benchmarks/mt-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github:latest" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/mt-bench:latest"]
}
