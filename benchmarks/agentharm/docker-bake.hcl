target "benchmark-agentharm" {
  context = "benchmarks/agentharm"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/agentharm:latest"]
}
