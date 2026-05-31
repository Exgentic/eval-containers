target "benchmark-code-contests" {
  context = "benchmarks/code-contests"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/code-contests:latest"]
}
