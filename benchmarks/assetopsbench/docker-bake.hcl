target "benchmark-assetopsbench" {
  context = "benchmarks/assetopsbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/assetopsbench:${TAG}"]
}
