target "edge" {
  context = "containers/core/edge"
  tags = ["${REGISTRY}/core/edge:${TAG}"]
}
