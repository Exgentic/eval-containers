target "benchmark-humanevalplus" {
  context = "containers/benchmarks/humanevalplus"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/humanevalplus:${TAG}"]
}
