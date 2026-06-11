target "benchmark-mle-bench" {
  context = "containers/benchmarks/mle-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/mle-bench:${TAG}"]
}
