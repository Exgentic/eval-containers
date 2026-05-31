variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-gemini-cli" {
  context = "agents/gemini-cli"
  contexts = {
    "${REGISTRY}/core/agent-base-node" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/gemini-cli:latest"]
}
