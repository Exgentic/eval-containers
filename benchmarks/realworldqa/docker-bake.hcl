variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-realworldqa" {
  context = "benchmarks/realworldqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/realworldqa:latest"]
}
