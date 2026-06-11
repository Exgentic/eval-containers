target "agent-bob" {
  context = "containers/agents/bob"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/bob:${TAG}"]
}
