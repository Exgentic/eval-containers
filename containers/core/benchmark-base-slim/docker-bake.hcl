target "benchmark-base-slim" {
  context = "containers/core/benchmark-base-slim"
  tags = ["${REGISTRY}/core/benchmark-base-slim:${TAG}"]
}
