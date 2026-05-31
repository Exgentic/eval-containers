target "litellm" {
  context = "core/litellm"
  tags = ["${REGISTRY}/core/litellm:${TAG}"]
}
