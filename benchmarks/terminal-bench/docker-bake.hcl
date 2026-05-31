variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-terminal-bench" {
  context = "benchmarks/terminal-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/terminal-bench:latest"]
}
