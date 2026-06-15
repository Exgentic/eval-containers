target "benchmark-base-hf" {
  context = "containers/core/benchmark-base-hf"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-hf:${TAG}"]
}
