variable "REGISTRY" { default = "quay.io/eval-containers" }

target "test-exact-match" {
  context = "core/test-exact-match"
  tags = ["${REGISTRY}/core/test-exact-match:latest"]
}
