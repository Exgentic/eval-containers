target "agent-terminus-2" {
  context = "containers/agents/terminus-2"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/terminus-2:${TAG}"]
}
