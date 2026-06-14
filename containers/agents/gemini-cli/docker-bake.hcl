target "agent-gemini-cli" {
  context = "containers/agents/gemini-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/gemini-cli:${TAG}"]
}
