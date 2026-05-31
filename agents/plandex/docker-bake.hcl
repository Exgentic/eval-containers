variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "PLANDEX_VERSION" { default = "2.2.1" }

target "agent-plandex" {
  context = "agents/plandex"
  contexts = {
    "${REGISTRY}/core/agent-base-rust:latest" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/plandex:${PLANDEX_VERSION}"]
}
