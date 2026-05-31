target "model-gpt-5" {
  context = "models/gpt-5"
  contexts = {
    "${REGISTRY}/core/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/gpt-5:latest"]
}
