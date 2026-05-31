target "benchmark-browsecomp" {
  context = "benchmarks/browsecomp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/browsecomp:latest"]
}
