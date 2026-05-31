variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "BOB_VERSION" { default = "1.0.1" }

target "agent-bob" {
  context = "agents/bob"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/bob:${BOB_VERSION}"]
}
