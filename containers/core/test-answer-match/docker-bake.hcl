target "test-answer-match" {
  context = "containers/core/test-answer-match"
  tags = ["${REGISTRY}/core/test-answer-match:${TAG}"]
}
