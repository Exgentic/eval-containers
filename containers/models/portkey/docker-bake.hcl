target "model-portkey" {
  context = "containers/models/portkey"
  contexts = {
    "${REGISTRY}/gateways/portkey" = "target:gateway-portkey"
  }
  tags = ["${REGISTRY}/models/portkey:${TAG}"]
}
