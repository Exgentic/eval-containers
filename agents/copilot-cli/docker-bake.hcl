variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "COPILOT_CLI_VERSION" { default = "1.0.24" }

target "agent-copilot-cli" {
  context = "agents/copilot-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/copilot-cli:${COPILOT_CLI_VERSION}"]
}
