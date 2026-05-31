target "bifrost" {
  context = "gateways/bifrost"
  tags = ["${REGISTRY}/gateways/bifrost:${TAG}"]
}
