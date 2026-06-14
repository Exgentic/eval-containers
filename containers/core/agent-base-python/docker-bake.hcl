target "agent-base-python" {
  context = "containers/core/agent-base-python"
  tags = ["${REGISTRY}/core/agent-base-python:${TAG}"]
}
