variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-advbench" {
  context = "benchmarks/advbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/advbench:latest"]
}
