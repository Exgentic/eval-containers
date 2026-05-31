variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "CONTINUE_CLI_VERSION" { default = "1.5.45" }

target "agent-continue-cli" {
  context = "agents/continue-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/continue-cli:${CONTINUE_CLI_VERSION}"]
}
