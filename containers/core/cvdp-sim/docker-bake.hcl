target "cvdp-sim" {
  context = "containers/core/cvdp-sim"
  tags    = ["${REGISTRY}/core/cvdp-sim:${TAG}"]
}
