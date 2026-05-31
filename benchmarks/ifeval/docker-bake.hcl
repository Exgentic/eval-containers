target "benchmark-ifeval" {
  context = "benchmarks/ifeval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ifeval:latest"]
}
