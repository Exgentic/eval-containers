variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "OPENCODE_VERSION" { default = "1.4.3" }

target "agent-opencode" {
  context = "agents/opencode"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/opencode:${OPENCODE_VERSION}"]
}
