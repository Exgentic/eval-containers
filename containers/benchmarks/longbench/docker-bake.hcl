target "benchmark-longbench" {
  context = "containers/benchmarks/longbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/longbench:${TAG}"]
}
