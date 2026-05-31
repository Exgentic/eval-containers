target "test-exact-match" {
  context = "core/test-exact-match"
  tags = ["${REGISTRY}/core/test-exact-match:${TAG}"]
}
