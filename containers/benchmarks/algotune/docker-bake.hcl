target "benchmark-algotune" {
  context = "containers/benchmarks/algotune"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/algotune:${TAG}"]
}
