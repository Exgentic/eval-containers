variable "HF_TOKEN" { default = "" }

target "benchmark-realworldqa" {
  context = "containers/benchmarks/realworldqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/realworldqa:${TAG}"]
}
