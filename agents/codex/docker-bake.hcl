variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "CODEX_VERSION" { default = "0.120.0" }

target "agent-codex" {
  context = "agents/codex"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/codex:${CODEX_VERSION}"]
}
