target "agent-openhands" {
  context = "agents/openhands"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/openhands:latest"]
}
