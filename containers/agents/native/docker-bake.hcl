target "agent-native" {
  context = "containers/agents/native"
  tags    = ["${REGISTRY}/agents/native:${TAG}"]
}
