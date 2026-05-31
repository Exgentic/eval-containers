target "benchmark-swe-gym" {
  context = "benchmarks/swe-gym"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/swe-gym:${TAG}"]
}
