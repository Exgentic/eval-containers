target "model-claude-sonnet-4" {
  context = "containers/models/claude-sonnet-4"
  contexts = {
    "${REGISTRY}/core/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/claude-sonnet-4:${TAG}"]
}
