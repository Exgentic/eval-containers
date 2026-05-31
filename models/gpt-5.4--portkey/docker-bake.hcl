variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-gpt-5_4--portkey" {
  context = "models/gpt-5.4--portkey"
  contexts = {
    "${REGISTRY}/gateways/portkey" = "target:portkey"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--portkey:latest"]
}
