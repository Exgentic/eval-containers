target "model-claude-opus-4" {
  context = "models/claude-opus-4"
  contexts = {
    "${REGISTRY}/core/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/claude-opus-4:${TAG}"]
}
