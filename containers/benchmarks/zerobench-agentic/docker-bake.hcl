target "benchmark-zerobench-agentic" {
  context = "containers/benchmarks/zerobench-agentic"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
    "${REGISTRY}/core/test-answer-match" = "target:test-answer-match"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/zerobench-agentic:${TAG}"]
}
