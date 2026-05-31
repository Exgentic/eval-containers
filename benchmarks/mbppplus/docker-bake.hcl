target "benchmark-mbppplus" {
  context = "benchmarks/mbppplus"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mbppplus:latest"]
}
