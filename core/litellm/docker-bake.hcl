variable "REGISTRY" { default = "quay.io/eval-containers" }

target "litellm" {
  context = "core/litellm"
  tags = ["${REGISTRY}/core/litellm:latest"]
}
