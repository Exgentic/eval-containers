target "agent-crush" {
  context = "containers/agents/crush"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/crush:${TAG}"]
}
