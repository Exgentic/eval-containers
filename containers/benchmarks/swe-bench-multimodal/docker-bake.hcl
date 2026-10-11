target "benchmark-swe-bench-multimodal" {
  context = "containers/benchmarks/swe-bench-multimodal"
  tags    = ["${REGISTRY}/benchmarks/swe-bench-multimodal:${TAG}"]
}
