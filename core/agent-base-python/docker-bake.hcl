variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-base-python" {
  context = "core/agent-base-python"
  tags = ["${REGISTRY}/core/agent-base-python:latest"]
}
