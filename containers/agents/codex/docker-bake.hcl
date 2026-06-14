target "agent-codex" {
  context = "containers/agents/codex"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/codex:${TAG}"]
}
