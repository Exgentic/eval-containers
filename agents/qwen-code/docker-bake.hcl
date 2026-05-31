variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "QWEN_CODE_VERSION" { default = "0.14.4" }

target "agent-qwen-code" {
  context = "agents/qwen-code"
  contexts = {
    "${REGISTRY}/core/agent-base-node:latest" = "target:agent-base-node"
  }
  tags = ["${REGISTRY}/agents/qwen-code:${QWEN_CODE_VERSION}"]
}
