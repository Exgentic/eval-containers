variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "GOOSE_VERSION" { default = "1.30.0" }

target "agent-goose" {
  context = "agents/goose"
  contexts = {
    "${REGISTRY}/core/agent-base-rust:latest" = "target:agent-base-rust"
  }
  tags = ["${REGISTRY}/agents/goose:${GOOSE_VERSION}"]
}
