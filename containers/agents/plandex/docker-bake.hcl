target "agent-plandex" {
  context = "containers/agents/plandex"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/plandex:${TAG}"]
}
