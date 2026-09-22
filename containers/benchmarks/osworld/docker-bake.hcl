target "benchmark-osworld" {
  context = "containers/benchmarks/osworld"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/osworld:${TAG}"]
}

# The desktop the agent drives. A sidecar image, not part of the eval image:
# the agent reaches it over HTTP, exactly as it reached the QEMU sidecar this
# replaces.
target "benchmark-osworld-desktop" {
  context    = "containers/benchmarks/osworld"
  dockerfile = "desktop.Dockerfile"
  tags       = ["${REGISTRY}/benchmarks/osworld-desktop:${TAG}"]
}
