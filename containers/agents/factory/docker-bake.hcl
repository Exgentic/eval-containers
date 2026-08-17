target "agent-factory" {
  context = "containers/agents/factory"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/factory:${TAG}"]
}
