target "benchmark-longbench" {
  context = "containers/benchmarks/longbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/longbench:${TAG}"]
}
