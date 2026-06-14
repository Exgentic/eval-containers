# The crane runner -- ONE generic image that materializes any (benchmark, agent)
# eval at run time, instead of pulling a pre-built evals/<b>--<a> combination
# image. It crane-exports the per-axis images to a rootfs and fuses them with
# bwrap/chroot: no Docker daemon, no DinD (proven daemonless -- see crane-poc.sh).
#
# STATUS: first cut. The fusion ([1]/[2] in `materialize`) is proven; wiring the
# in-rootfs gateway/otel via process-compose and the root-only grader perms is
# the remaining build-out + validation gate (see README.md). Nothing else in the
# repo depends on this image -- `--mode crane` opts in.
FROM debian:stable-slim

# Userspace image tool (no daemon) + the sandbox. crane exports an image's
# filesystem; bwrap/chroot runs a process inside it.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl bubblewrap jq tar \
 && rm -rf /var/lib/apt/lists/*

# Pin crane for reproducibility (override with --build-arg CRANE_VERSION=...).
ARG CRANE_VERSION=v0.20.3
RUN arch="$(dpkg --print-architecture)"; case "$arch" in amd64) arch=x86_64 ;; esac; \
    curl -fsSL "https://github.com/google/go-containerregistry/releases/download/${CRANE_VERSION}/go-containerregistry_Linux_${arch}.tar.gz" \
      | tar -xz -C /usr/local/bin crane

COPY materialize /usr/local/bin/materialize
RUN chmod +x /usr/local/bin/materialize
ENTRYPOINT ["/usr/local/bin/materialize"]
