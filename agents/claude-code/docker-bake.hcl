variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "CLAUDE_CODE_VERSION" { default = "2.1.104" }

target "agent-claude-code" {
  context = "agents/claude-code"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/claude-code:${CLAUDE_CODE_VERSION}"]
}
