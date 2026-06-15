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
COPY --chmod=755 start /opt/gateway/start
COPY --chmod=755 health /opt/gateway/health

EXPOSE 4000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 CMD ["/opt/gateway/server", "health"]
CMD ["/opt/gateway/server"]
