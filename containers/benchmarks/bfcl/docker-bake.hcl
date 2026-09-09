target "benchmark-bfcl" {
  context = "containers/benchmarks/bfcl"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/bfcl:${TAG}"]
}
