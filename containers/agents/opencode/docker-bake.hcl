target "agent-opencode" {
  context = "containers/agents/opencode"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/opencode:${TAG}"]
}
