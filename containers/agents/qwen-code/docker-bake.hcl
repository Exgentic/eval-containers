target "agent-qwen-code" {
  context = "containers/agents/qwen-code"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/qwen-code:${TAG}"]
}
