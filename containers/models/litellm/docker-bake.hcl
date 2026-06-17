target "model-litellm" {
  context = "containers/models/litellm"
  contexts = {
    "${REGISTRY}/gateways/litellm" = "target:gateway-litellm"
  }
  tags = ["${REGISTRY}/models/litellm:${TAG}"]
}
