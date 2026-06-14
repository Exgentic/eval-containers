variable "HF_TOKEN" { default = "" }

target "benchmark-base-hf" {
  context = "containers/core/benchmark-base-hf"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/core/benchmark-base-hf:${TAG}"]
}
