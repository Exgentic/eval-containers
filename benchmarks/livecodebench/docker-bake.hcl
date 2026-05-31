target "benchmark-livecodebench" {
  context = "benchmarks/livecodebench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/livecodebench:${TAG}"]
}
