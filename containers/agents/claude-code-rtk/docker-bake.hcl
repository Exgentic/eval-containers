target "agent-claude-code-rtk" {
  context = "containers/agents/claude-code-rtk"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/claude-code-rtk:${TAG}"]
}
