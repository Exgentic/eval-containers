target "benchmark-swe-bench-multimodal" {
  context = "containers/benchmarks/swe-bench-multimodal"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/swe-bench-multimodal:${TAG}"]
}
