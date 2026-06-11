target "test-exact-match" {
  context = "containers/core/test-exact-match"
  tags = ["${REGISTRY}/core/test-exact-match:${TAG}"]
}
