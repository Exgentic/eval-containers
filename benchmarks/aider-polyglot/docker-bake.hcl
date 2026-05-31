variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-aider-polyglot" {
  context = "benchmarks/aider-polyglot"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/aider-polyglot:latest"]
}
