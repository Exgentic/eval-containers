target "agent-goose" {
  context = "containers/agents/goose"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/goose:${TAG}"]
}
