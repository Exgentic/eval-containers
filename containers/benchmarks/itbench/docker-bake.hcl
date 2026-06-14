target "benchmark-itbench" {
  context = "containers/benchmarks/itbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/itbench:${TAG}"]
}
