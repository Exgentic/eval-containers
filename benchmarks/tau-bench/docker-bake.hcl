variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-tau-bench" {
  context = "benchmarks/tau-bench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/tau-bench:latest"]
}
