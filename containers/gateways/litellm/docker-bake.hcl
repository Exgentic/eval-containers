target "gateway-litellm" {
  context = "containers/gateways/litellm"
  tags = ["${REGISTRY}/gateways/litellm:${TAG}"]
}
