target "benchmark-assertllm2-verify" {
  context = "containers/benchmarks/assertllm2-verify"
  contexts = {
    "${REGISTRY}/core/assertllm2-harness" = "target:assertllm2-harness"
  }
  tags = ["${REGISTRY}/benchmarks/assertllm2-verify:${TAG}"]
}
