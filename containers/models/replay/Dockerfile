# Replay model: a tiny Rust binary that serves recorded OTLP gen_ai traces as
# live provider APIs (models/RULES.md rule 17) — no Python/flask, no API keys.
# A ~5 MB binary on distroless replaces the ~150 MB python+flask image.
#
# /opt/gateway is the gateway contract: the compose/k8s sidecar runs the binary
# directly; the standalone bundle COPYs /opt/gateway and execs /opt/gateway/start
# in-process (under a benchmark base that has a shell). The distroless final
# image has no shell, so its CMD/HEALTHCHECK exec the binary directly, and the
# binary carries its own `health` probe (`replay-server health`).

FROM rust:1-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# Test then build — gates the image on the unit suite (this crate is standalone,
# so it is not covered by the workspace `cargo test`).
RUN cargo test --release && cargo build --release

# distroless/cc carries glibc + CA certs but no shell — fine for a self-contained
# binary that only reads /data and listens on :4000. `:nonroot` runs as an
# unprivileged uid (the binary needs no root: port 4000 is unprivileged and the
# fixture is a world-readable :ro mount).
FROM gcr.io/distroless/cc-debian12:nonroot

LABEL eval.type="model"
LABEL eval.model.name="replay"
LABEL eval.model.provider="replay"

COPY --from=build /src/target/release/replay-server /opt/gateway/server
# /opt/gateway/health is the SAME binary, not a shell script: the binary detects
# the health-probe invocation by its argv[0] basename (`health`). This makes
# replay a true drop-in for the gateway slot — services.yaml's default
# healthcheck `["CMD", "/opt/gateway/health"]` (the contract bifrost's shell
# `health` honors) passes here too, with no per-test healthcheck override. The
# binary form `["/opt/gateway/server", "health"]` still works (explicit arg).
COPY --from=build /src/target/release/replay-server /opt/gateway/health
# start is the shell entrypoint for the standalone bundle (run under a
# shell-bearing base); it execs the binary. Not runnable on this distroless image.
COPY --chmod=755 start /opt/gateway/start

EXPOSE 4000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 CMD ["/opt/gateway/health"]
CMD ["/opt/gateway/server"]
