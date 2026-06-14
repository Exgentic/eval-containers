target "benchmark-appworld" {
  context = "containers/benchmarks/appworld"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/appworld:${TAG}"]
}
