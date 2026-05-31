variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-longbench" {
  context = "benchmarks/longbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/longbench:latest"]
}
