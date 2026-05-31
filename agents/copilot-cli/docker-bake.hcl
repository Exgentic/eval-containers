target "agent-copilot-cli" {
  context = "agents/copilot-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/copilot-cli:${TAG}"]
}
