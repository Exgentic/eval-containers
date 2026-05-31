target "agent-openclaw" {
  context = "agents/openclaw"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/openclaw:latest"]
}
