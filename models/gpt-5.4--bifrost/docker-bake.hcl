variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-gpt-5_4--bifrost" {
  context = "models/gpt-5.4--bifrost"
  contexts = {
    "${REGISTRY}/gateways/bifrost:latest" = "target:bifrost"
  }
  tags = ["${REGISTRY}/models/gpt-5.4--bifrost:latest"]
}
