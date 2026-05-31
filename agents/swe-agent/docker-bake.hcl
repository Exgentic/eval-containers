target "agent-swe-agent" {
  context = "agents/swe-agent"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/swe-agent:latest"]
}
