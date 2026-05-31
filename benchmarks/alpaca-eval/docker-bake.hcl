target "benchmark-alpaca-eval" {
  context = "benchmarks/alpaca-eval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/alpaca-eval:${TAG}"]
}
