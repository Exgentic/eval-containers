target "benchmark-healthbench" {
  context = "containers/benchmarks/healthbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/healthbench:${TAG}"]
}
