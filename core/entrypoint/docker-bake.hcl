target "entrypoint" {
  context = "core/entrypoint"
  tags = ["${REGISTRY}/core/entrypoint:${TAG}"]
}
