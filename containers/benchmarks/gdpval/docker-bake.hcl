target "benchmark-gdpval" {
  context = "containers/benchmarks/gdpval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/gdpval:${TAG}"]
}
