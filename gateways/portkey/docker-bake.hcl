variable "REGISTRY" { default = "quay.io/eval-containers" }

target "portkey" {
  context = "gateways/portkey"
  tags = ["${REGISTRY}/gateways/portkey:latest"]
}
