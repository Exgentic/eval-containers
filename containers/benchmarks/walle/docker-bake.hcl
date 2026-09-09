target "benchmark-walle" {
  context = "containers/benchmarks/walle"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/walle:${TAG}"]
}
