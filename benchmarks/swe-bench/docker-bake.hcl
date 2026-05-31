variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-swe-bench" {
  context = "benchmarks/swe-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/swe-bench:latest"]
}
