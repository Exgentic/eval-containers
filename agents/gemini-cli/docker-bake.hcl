variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "GEMINI_CLI_VERSION" { default = "0.37.2" }

target "agent-gemini-cli" {
  context = "agents/gemini-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/gemini-cli:${GEMINI_CLI_VERSION}"]
}
