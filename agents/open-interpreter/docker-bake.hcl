variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "OPEN_INTERPRETER_VERSION" { default = "0.4.3" }

target "agent-open-interpreter" {
  context = "agents/open-interpreter"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/open-interpreter:${OPEN_INTERPRETER_VERSION}"]
}
