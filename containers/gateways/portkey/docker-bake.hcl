target "gateway-portkey" {
  context = "containers/gateways/portkey"
  tags = ["${REGISTRY}/gateways/portkey:${TAG}"]
}
