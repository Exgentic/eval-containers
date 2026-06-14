target "benchmark-livecodebench" {
  context = "containers/benchmarks/livecodebench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/livecodebench:${TAG}"]
}
