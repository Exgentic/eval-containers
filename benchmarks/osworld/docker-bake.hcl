target "benchmark-osworld" {
  context = "benchmarks/osworld"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/osworld:${TAG}"]
}
