variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mrcr" {
  context = "benchmarks/mrcr"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mrcr:latest"]
}
