variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-compilebench" {
  context = "benchmarks/compilebench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/compilebench:latest"]
}
