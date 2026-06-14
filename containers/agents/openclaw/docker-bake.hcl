target "agent-openclaw" {
  context = "containers/agents/openclaw"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/openclaw:${TAG}"]
}
