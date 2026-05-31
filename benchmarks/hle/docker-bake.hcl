variable "HF_TOKEN" { default = "" }

target "benchmark-hle" {
  context = "benchmarks/hle"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/hle:latest"]
}
