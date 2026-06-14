target "benchmark-wmdp" {
  context = "containers/benchmarks/wmdp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/wmdp:${TAG}"]
}
