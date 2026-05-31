variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "AIDER_VERSION" { default = "0.86.2" }

target "agent-aider" {
  context = "agents/aider"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/aider:${AIDER_VERSION}"]
}
