variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-appworld" {
  context = "benchmarks/appworld"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/appworld:latest"]
}
