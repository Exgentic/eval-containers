variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mle-bench" {
  context = "benchmarks/mle-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/mle-bench:latest"]
}
