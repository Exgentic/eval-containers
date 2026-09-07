target "benchmark-hwe-bench" {
  context = "containers/benchmarks/hwe-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/hwe-bench:${TAG}"]
}
