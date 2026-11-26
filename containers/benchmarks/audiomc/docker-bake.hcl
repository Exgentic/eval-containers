target "benchmark-audiomc" {
  context = "containers/benchmarks/audiomc"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/audiomc:${TAG}"]
}
