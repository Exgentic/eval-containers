target "benchmark-osworld2" {
  context = "containers/benchmarks/osworld2"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  # The 108 task classes are gated; the token is read inside the RUN and never
  # written to a layer (benchmarks/RULES.md 8a).
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/osworld2:${TAG}"]
}
