target "agent-ra-aid" {
  context = "agents/ra-aid"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/ra-aid:latest"]
}
