target "gateway-bifrost" {
  context = "containers/gateways/bifrost"
  tags = ["${REGISTRY}/gateways/bifrost:${TAG}"]
}
