target "agent-claude-code" {
  context = "agents/claude-code"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/claude-code:latest"]
}
