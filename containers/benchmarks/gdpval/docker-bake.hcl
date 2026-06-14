variable "HF_TOKEN" { default = "" }

target "benchmark-gdpval" {
  context = "containers/benchmarks/gdpval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/gdpval:${TAG}"]
}
