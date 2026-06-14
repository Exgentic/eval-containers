target "benchmark-mrcr" {
  context = "containers/benchmarks/mrcr"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mrcr:${TAG}"]
}
