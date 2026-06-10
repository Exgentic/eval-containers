target "gateway-litellm" {
  context = "gateways/litellm"
  tags = ["${REGISTRY}/gateways/litellm:${TAG}"]
}
