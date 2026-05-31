variable "REGISTRY" { default = "quay.io/eval-containers" }
variable "HF_TOKEN" { default = "" }

target "benchmark-gdpval" {
  context = "benchmarks/gdpval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  args = { HF_TOKEN = HF_TOKEN }
  tags = ["${REGISTRY}/benchmarks/gdpval:latest"]
}
