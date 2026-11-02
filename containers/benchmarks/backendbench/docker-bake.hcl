target "benchmark-backendbench" {
  context = "containers/benchmarks/backendbench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/backendbench:${TAG}"]
}
