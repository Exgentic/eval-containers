target "agent-open-interpreter" {
  context = "containers/agents/open-interpreter"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/open-interpreter:${TAG}"]
}
