variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-swe-bench-pro" {
  context = "benchmarks/swe-bench-pro"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/swe-bench-pro:latest"]
}
