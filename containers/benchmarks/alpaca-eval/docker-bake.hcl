target "benchmark-alpaca-eval" {
  context = "containers/benchmarks/alpaca-eval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/alpaca-eval:${TAG}"]
}
