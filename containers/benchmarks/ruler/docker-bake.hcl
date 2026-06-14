target "benchmark-ruler" {
  context = "containers/benchmarks/ruler"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ruler:${TAG}"]
}
