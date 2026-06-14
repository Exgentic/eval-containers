target "entrypoint" {
  context = "containers/core/entrypoint"
  tags = ["${REGISTRY}/core/entrypoint:${TAG}"]
}
