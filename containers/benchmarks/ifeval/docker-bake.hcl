target "benchmark-ifeval" {
  context = "containers/benchmarks/ifeval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ifeval:${TAG}"]
}
