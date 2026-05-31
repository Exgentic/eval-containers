variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-plandex" {
  context = "agents/plandex"
  contexts = {
    "${REGISTRY}/core/agent-base-rust" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/plandex:latest"]
}
