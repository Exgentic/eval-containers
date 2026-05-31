variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-aider" {
  context = "agents/aider"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/aider:latest"]
}
