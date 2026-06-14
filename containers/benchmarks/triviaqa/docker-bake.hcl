target "benchmark-triviaqa" {
  context = "containers/benchmarks/triviaqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/triviaqa:${TAG}"]
}
