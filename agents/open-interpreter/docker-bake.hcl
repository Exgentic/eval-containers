variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-open-interpreter" {
  context = "agents/open-interpreter"
  contexts = {
    "${REGISTRY}/core/agent-base-python" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/open-interpreter:latest"]
}
