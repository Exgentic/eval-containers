variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "RA_AID_VERSION" { default = "0.19.1" }

target "agent-ra-aid" {
  context = "agents/ra-aid"
  contexts = {
    "${REGISTRY}/core/agent-base-python:latest" = "target:agent-base-python"
  }
  tags = ["${REGISTRY}/agents/ra-aid:${RA_AID_VERSION}"]
}
