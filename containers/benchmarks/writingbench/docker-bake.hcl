target "benchmark-writingbench" {
  context = "containers/benchmarks/writingbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/writingbench:${TAG}"]
}
