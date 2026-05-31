variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-naturalquestions" {
  context = "benchmarks/naturalquestions"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/naturalquestions:latest"]
}
