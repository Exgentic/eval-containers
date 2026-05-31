variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-base-github" {
  context = "core/benchmark-base-github"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-github:latest"]
}
