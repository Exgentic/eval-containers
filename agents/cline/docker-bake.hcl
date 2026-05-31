variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "CLINE_VERSION" { default = "2.15.0" }

target "agent-cline" {
  context = "agents/cline"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/cline:${CLINE_VERSION}"]
}
