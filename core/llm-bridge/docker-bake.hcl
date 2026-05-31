variable "REGISTRY" { default = "quay.io/eval-containers" }

target "llm-bridge" {
  context = "core/llm-bridge"
  tags = ["${REGISTRY}/core/llm-bridge:latest"]
}
