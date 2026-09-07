target "agent-mock" {
  context = "containers/agents/mock"
  tags    = ["${REGISTRY}/agents/mock:${TAG}"]
}
