variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "SWE_AGENT_VERSION" { default = "1.1.0" }

target "agent-swe-agent" {
  context = "agents/swe-agent"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/swe-agent:${SWE_AGENT_VERSION}"]
}
