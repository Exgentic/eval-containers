target "agent-mini-swe-agent" {
  context = "containers/agents/mini-swe-agent"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/mini-swe-agent:${TAG}"]
}
