variable "REGISTRY" { default = "quay.io/eval-containers" }

target "agent-base-rust" {
  context = "core/agent-base-rust"
  tags = ["${REGISTRY}/core/agent-base-rust:latest"]
}
