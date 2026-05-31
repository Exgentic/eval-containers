variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-compilebench" {
  context = "benchmarks/compilebench"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/compilebench:latest"]
}
