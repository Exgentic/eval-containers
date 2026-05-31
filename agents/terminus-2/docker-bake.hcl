variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-terminus-2" {
  context = "agents/terminus-2"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/terminus-2:latest"]
}
