target "benchmark-browsecomp" {
  context = "containers/benchmarks/browsecomp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/browsecomp:${TAG}"]
}
