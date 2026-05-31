variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-gpt-5" {
  context = "models/gpt-5"
  contexts = {
    "${REGISTRY}/core/litellm:latest" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/gpt-5:latest"]
}
