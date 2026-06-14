target "benchmark-enterpriseops-gym" {
  context = "containers/benchmarks/enterpriseops-gym"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/enterpriseops-gym:${TAG}"]
}
