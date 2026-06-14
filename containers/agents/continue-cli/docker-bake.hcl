target "agent-continue-cli" {
  context = "containers/agents/continue-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/continue-cli:${TAG}"]
}
