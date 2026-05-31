variable "REGISTRY" { default = "quay.io/eval-containers" }

target "runtime-bundle" {
  context = "core/runtime-bundle"
  tags = ["${REGISTRY}/core/runtime-bundle:latest"]
}
