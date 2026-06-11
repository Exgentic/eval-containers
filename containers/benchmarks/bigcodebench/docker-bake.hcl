target "benchmark-bigcodebench" {
  context = "containers/benchmarks/bigcodebench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/bigcodebench:${TAG}"]
}
