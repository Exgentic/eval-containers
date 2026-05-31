variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-mini-swe-agent" {
  context = "agents/mini-swe-agent"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/mini-swe-agent:latest"]
}
