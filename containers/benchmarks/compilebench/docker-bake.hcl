target "benchmark-compilebench" {
  context = "containers/benchmarks/compilebench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/compilebench:${TAG}"]
}
