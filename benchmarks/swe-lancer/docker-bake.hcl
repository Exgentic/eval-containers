variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-swe-lancer" {
  context = "benchmarks/swe-lancer"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/swe-lancer:latest"]
}
