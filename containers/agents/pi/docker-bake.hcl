target "agent-pi" {
  context = "containers/agents/pi"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/pi:${TAG}"]
}
