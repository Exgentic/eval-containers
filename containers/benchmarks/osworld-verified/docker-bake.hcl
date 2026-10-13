target "benchmark-osworld-verified" {
  context = "containers/benchmarks/osworld-verified"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/osworld-verified:${TAG}"]
}
