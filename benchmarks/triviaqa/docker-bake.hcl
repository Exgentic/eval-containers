target "benchmark-triviaqa" {
  context = "benchmarks/triviaqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/triviaqa:${TAG}"]
}
