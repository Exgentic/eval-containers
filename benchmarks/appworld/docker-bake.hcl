target "benchmark-appworld" {
  context = "benchmarks/appworld"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/appworld:${TAG}"]
}
