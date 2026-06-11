target "llm-bridge" {
  context = "containers/core/llm-bridge"
  tags = ["${REGISTRY}/core/llm-bridge:${TAG}"]
}
