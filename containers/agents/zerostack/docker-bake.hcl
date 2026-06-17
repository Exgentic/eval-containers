target "agent-zerostack" {
  context = "containers/agents/zerostack"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/zerostack:${TAG}"]
}
