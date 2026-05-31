variable "REGISTRY" { default = "quay.io/eval-containers" }

target "bifrost" {
  context = "gateways/bifrost"
  tags = ["${REGISTRY}/gateways/bifrost:latest"]
}
