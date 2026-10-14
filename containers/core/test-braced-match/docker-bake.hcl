target "test-braced-match" {
  context = "containers/core/test-braced-match"
  tags = ["${REGISTRY}/core/test-braced-match:${TAG}"]
}
