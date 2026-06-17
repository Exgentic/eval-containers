target "model-bifrost" {
  context = "containers/models/bifrost"
  contexts = {
    "${REGISTRY}/gateways/bifrost" = "target:gateway-bifrost"
  }
  tags = ["${REGISTRY}/models/bifrost:${TAG}"]
}
