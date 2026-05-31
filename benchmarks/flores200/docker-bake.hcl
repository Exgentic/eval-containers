variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-flores200" {
  context = "benchmarks/flores200"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/flores200:latest"]
}
