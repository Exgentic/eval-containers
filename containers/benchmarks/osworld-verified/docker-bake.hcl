target "benchmark-osworld-verified" {
  context = "containers/benchmarks/osworld-verified"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/osworld-verified:${TAG}"]
}
