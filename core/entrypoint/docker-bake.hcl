variable "REGISTRY" { default = "quay.io/eval-containers" }

target "entrypoint" {
  context = "core/entrypoint"
  tags = ["${REGISTRY}/core/entrypoint:latest"]
}
