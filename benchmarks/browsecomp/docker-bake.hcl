variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-browsecomp" {
  context = "benchmarks/browsecomp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/browsecomp:latest"]
}
