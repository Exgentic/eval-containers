variable "REGISTRY" { default = "quay.io/eval-containers" }

target "litellm" {
  context = "gateways/litellm"
  tags = ["${REGISTRY}/gateways/litellm:latest"]
}
