target "gosu" {
  context = "containers/core/gosu"
  tags = ["${REGISTRY}/core/gosu:${TAG}"]
}
