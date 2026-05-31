target "model-gpt-5_4--litellm" {
  context = "models/gpt-5.4--litellm"
  contexts = {
    "${REGISTRY}/gateways/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--litellm:${TAG}"]
}
