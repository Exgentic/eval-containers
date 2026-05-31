variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "OPENCLAW_VERSION" { default = "2026.4.11" }

target "agent-openclaw" {
  context = "agents/openclaw"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/openclaw:${OPENCLAW_VERSION}"]
}
