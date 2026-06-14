target "benchmark-workarena" {
  context = "containers/benchmarks/workarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/workarena:${TAG}"]
}
