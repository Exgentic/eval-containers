target "benchmark-naturalquestions" {
  context = "benchmarks/naturalquestions"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/naturalquestions:${TAG}"]
}
