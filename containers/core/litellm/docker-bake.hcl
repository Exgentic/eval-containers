target "litellm" {
  context = "containers/core/litellm"
  tags = ["${REGISTRY}/core/litellm:${TAG}"]
}
