target "agent-goose" {
  context = "agents/goose"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/goose:${TAG}"]
}
