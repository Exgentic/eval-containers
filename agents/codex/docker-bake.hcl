target "agent-codex" {
  context = "agents/codex"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/codex:latest"]
}
