target "agent-plandex" {
  context = "containers/agents/plandex"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/plandex:${TAG}"]
}
