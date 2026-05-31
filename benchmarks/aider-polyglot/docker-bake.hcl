target "benchmark-aider-polyglot" {
  context = "benchmarks/aider-polyglot"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/aider-polyglot:${TAG}"]
}
