target "agent-cline" {
  context = "agents/cline"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/cline:latest"]
}
