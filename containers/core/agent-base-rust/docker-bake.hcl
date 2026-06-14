target "agent-base-rust" {
  context = "containers/core/agent-base-rust"
  tags = ["${REGISTRY}/core/agent-base-rust:${TAG}"]
}
