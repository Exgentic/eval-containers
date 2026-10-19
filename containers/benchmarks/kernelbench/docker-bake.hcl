target "benchmark-kernelbench" {
  context = "containers/benchmarks/kernelbench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/kernelbench:${TAG}"]
}
