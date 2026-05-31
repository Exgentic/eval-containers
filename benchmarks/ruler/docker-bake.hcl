target "benchmark-ruler" {
  context = "benchmarks/ruler"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ruler:${TAG}"]
}
