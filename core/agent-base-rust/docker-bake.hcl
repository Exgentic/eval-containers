target "agent-base-rust" {
  context = "core/agent-base-rust"
  tags = ["${REGISTRY}/core/agent-base-rust:${TAG}"]
}
