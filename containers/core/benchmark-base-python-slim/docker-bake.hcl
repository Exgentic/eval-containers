target "benchmark-base-python-slim" {
  context = "containers/core/benchmark-base-python-slim"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-python-slim:${TAG}"]
}
