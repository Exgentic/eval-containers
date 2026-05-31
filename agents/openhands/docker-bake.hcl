variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "OPENHANDS_VERSION" { default = "1.7.0" }

target "agent-openhands" {
  context = "agents/openhands"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/openhands:${OPENHANDS_VERSION}"]
}
