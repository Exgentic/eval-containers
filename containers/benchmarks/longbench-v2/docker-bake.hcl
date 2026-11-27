target "benchmark-longbench-v2" {
  context = "containers/benchmarks/longbench-v2"
  contexts = {
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/longbench-v2:${TAG}"]
}
