target "model-gpt-5_4--bifrost" {
  context = "containers/models/gpt-5.4--bifrost"
  contexts = {
    "${REGISTRY}/gateways/bifrost" = "target:gateway-bifrost"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--bifrost:${TAG}"]
}
