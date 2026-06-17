target "benchmark-base-slim" {
  context = "containers/core/benchmark-base-slim"
  contexts = {
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/core/benchmark-base-slim:${TAG}"]
}
