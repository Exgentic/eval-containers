variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-gaia" {
  context = "benchmarks/gaia"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/gaia:latest"]
}
