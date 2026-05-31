target "benchmark-base-external" {
  context = "core/benchmark-base-external"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-external:latest"]
}
