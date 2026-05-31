target "model-gpt-4_1-mini" {
  context = "models/gpt-4.1-mini"
  contexts = {
    "${REGISTRY}/core/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/gpt-4.1-mini:latest"]
}
