target "agent-aider" {
  context = "containers/agents/aider"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/aider:${TAG}"]
}
