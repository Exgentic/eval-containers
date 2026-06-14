target "benchmark-alpaca-eval" {
  context = "containers/benchmarks/alpaca-eval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/alpaca-eval:${TAG}"]
}
