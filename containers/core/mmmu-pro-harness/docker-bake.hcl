target "mmmu-pro-harness" {
  context = "containers/core/mmmu-pro-harness"
  tags = ["${REGISTRY}/core/mmmu-pro-harness:${TAG}"]
}
