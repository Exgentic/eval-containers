variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "TERMINUS_2_VERSION" { default = "0.6.4" }

target "agent-terminus-2" {
  context = "agents/terminus-2"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/terminus-2:${TERMINUS_2_VERSION}"]
}
