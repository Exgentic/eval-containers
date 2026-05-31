variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-claude-sonnet-4" {
  context = "models/claude-sonnet-4"
  contexts = {
    "${REGISTRY}/core/litellm:latest" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/claude-sonnet-4:latest"]
}
