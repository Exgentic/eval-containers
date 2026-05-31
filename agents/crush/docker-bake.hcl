variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "CRUSH_VERSION" { default = "0.57.0" }

target "agent-crush" {
  context = "agents/crush"
  contexts = {
    "${REGISTRY}/core/agent-base-rust:latest" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/crush:${CRUSH_VERSION}"]
}
