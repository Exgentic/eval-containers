variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "MINI_SWE_AGENT_VERSION" { default = "2.2.8" }

target "agent-mini-swe-agent" {
  context = "agents/mini-swe-agent"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/mini-swe-agent:${MINI_SWE_AGENT_VERSION}"]
}
