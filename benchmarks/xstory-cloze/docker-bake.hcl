variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-xstory-cloze" {
  context = "benchmarks/xstory-cloze"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match:latest" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/xstory-cloze:latest"]
}
