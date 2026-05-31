target "benchmark-writingbench" {
  context = "benchmarks/writingbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/writingbench:latest"]
}
