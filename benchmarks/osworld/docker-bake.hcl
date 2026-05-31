variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-osworld" {
  context = "benchmarks/osworld"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/osworld:latest"]
}
