target "agent-continue-cli" {
  context = "agents/continue-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/continue-cli:${TAG}"]
}
