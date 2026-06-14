target "model-gpt-5_4--litellm" {
  context = "containers/models/gpt-5.4--litellm"
  contexts = {
    "${REGISTRY}/gateways/litellm" = "target:gateway-litellm"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--litellm:${TAG}"]
}
