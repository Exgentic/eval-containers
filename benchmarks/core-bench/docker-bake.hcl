variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-core-bench" {
  context = "benchmarks/core-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/core-bench:latest"]
}
