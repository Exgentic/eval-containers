target "benchmark-assertllm2-verify-sim" {
  context = "containers/benchmarks/assertllm2-verify-sim"
  contexts = {
    "${REGISTRY}/core/assertllm2-harness" = "target:assertllm2-harness"
  }
  tags = ["${REGISTRY}/benchmarks/assertllm2-verify-sim:${TAG}"]
}
