target "model-gpt-5_4--portkey" {
  context = "models/gpt-5.4--portkey"
  contexts = {
    "${REGISTRY}/gateways/portkey" = "target:gateway-portkey"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--portkey:${TAG}"]
}
