target "benchmark-healthbench" {
  context = "containers/benchmarks/healthbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/healthbench:${TAG}"]
}
