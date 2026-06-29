//! Gateway invariant suite — pins the assumptions we have on each of
//! the three gateway flavors (bifrost, litellm, portkey).
//!
//! Three buckets of tests:
//!
//!   1. **Static (no runtime, no creds)** — read the Dockerfile text /
//!      filesystem and assert the structural invariants: label values,
//!      version pins, absence of stripped components (e.g. the bifrost
//!      sidecar that portkey no longer bundles). Fastest checks, run
//!      every CI build.
//!
//!   2. **Boot + protocol matrix (runtime, no creds)** — start each
//!      gateway image via testcontainers, hit its health endpoint, and
//!      for portkey assert that /anthropic and /genai return 501 with
//!      a structured `error.type=not_implemented` body. These don't
//!      hit upstream — Caddy short-circuits before any LLM call.
//!
//!   3. **Upstream + OTel emission (#[ignore])** — start a gateway
//!      next to an otelcol sidecar on a shared network, with a bind-
//!      mounted /output. Fire a real chat completion against the
//!      upstream from `.env`, assert the gateway both serves the
//!      protocol and emits gen_ai.* OTel semconv spans to
//!      /output/traces.jsonl. litellm additionally must write
//!      trajectory.jsonl + result.json via its eval_logger callback;
//!      portkey is asserted NOT to emit gateway-side spans on
//!      /openai (documents the known limitation — see
//!      gateways/portkey/start).
//!
//! ## Why testcontainers, not bash
//!
//! tests/RULES.md rule 6 forbids `Command::new("docker")` for container
//! lifecycle — every container must go through the testcontainers
//! library. This file is the canonical example: `GenericImage` for the
//! single-container boot/protocol tests, plus a shared-network pair
//! (otelcol + gateway) for OTel verification. The library auto-cleans
//! containers and networks on test exit (or panic).
//!
//! ## Run
//!
//!   cargo test --test gateways                       # static + no-creds (≈1 min cold)
//!   cargo test --test gateways -- --ignored          # full upstream + OTel matrix
//!
//! `cargo test --test gateways -- --ignored` requires `.env` with
//! OPENAI_API_KEY + OPENAI_API_BASE pointing at a working upstream
//! (any OpenAI-compatible endpoint that serves the configured model
//! — IBM litellm, OpenAI, Azure, vLLM, etc.).

use std::path::Path;
use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::sync::OnceCell;

#[path = "../common/mod.rs"]
mod common;

// ─── Constants ───────────────────────────────────────────────────────

/// The three gateway flavors covered by this suite. Adding a flavor:
/// add it here, add a Dockerfile under `gateways/<flavor>/`, and the
/// static checks will pick it up automatically.
const FLAVORS: &[&str] = &["bifrost", "litellm", "portkey"];

/// The /health-style probe paths differ per flavor (no shared
/// convention — bifrost: /api/health, litellm: /health/liveness,
/// portkey: /). Used as the readiness probe in `start_gateway`.
fn health_path(flavor: &str) -> &'static str {
    match flavor {
        "bifrost" => "/api/health",
        "litellm" => "/health/liveness",
        "portkey" => "/",
        _ => panic!("unknown flavor: {flavor}"),
    }
}

fn dockerfile_text(flavor: &str) -> String {
    std::fs::read_to_string(
        test_support::repo_root().join(format!("containers/gateways/{flavor}/Dockerfile")),
    )
    .unwrap_or_else(|e| panic!("read gateways/{flavor}/Dockerfile: {e}"))
}

fn gateway_image_ref(flavor: &str) -> (String, String) {
    (
        format!("ghcr.io/exgentic/models/{flavor}"),
        "latest".to_string(),
    )
}

// ─── Build bootstrap ─────────────────────────────────────────────────
//
// Per tests/RULES.md rule 6c, image builds shell to `docker buildx bake`
// (the framework's canonical build path) via `tests/run/common/mod.rs`.
// RUN/START/STOP still go through testcontainers-rs per rule 6.

static IMAGES_BUILT: OnceCell<()> = OnceCell::const_new();

/// Build every image these tests transitively need, exactly once per
/// process. Bake handles dep ordering via each artifact's `contexts`
/// (otelcol is a leaf; gateway/{flavor} is a leaf; model-{flavor}
/// depends on the matching gateway target).
async fn ensure_built() {
    IMAGES_BUILT
        .get_or_init(|| async {
            let _ = dotenvy::dotenv();
            let mut targets: Vec<String> = vec!["otel".to_string()];
            for f in FLAVORS {
                targets.push(format!("gateway-{f}"));
                targets.push(format!("model-{f}"));
            }
            let refs: Vec<&str> = targets.iter().map(String::as_str).collect();
            common::bake_targets(&refs).await;
        })
        .await;
}

// ─── Container start helpers ─────────────────────────────────────────

/// Start a single gateway with the boot defaults. `extra_env` is
/// layered after the defaults so callers can override `OPENAI_API_KEY`
/// + `OPENAI_API_BASE` to point at a real upstream.
async fn start_gateway(flavor: &str, extra_env: &[(&str, &str)]) -> ContainerAsync<GenericImage> {
    ensure_built().await;
    let (name, tag) = gateway_image_ref(flavor);

    let mut img = GenericImage::new(name, tag)
        .with_exposed_port(ContainerPort::Tcp(4000))
        .with_wait_for(WaitFor::Http(Box::new(
            HttpWaitStrategy::new(health_path(flavor))
                .with_port(ContainerPort::Tcp(4000))
                .with_poll_interval(Duration::from_millis(500))
                // `with_expected_status_code` is required by
                // testcontainers 0.27 — strategy with no matcher errors
                // at start time. 200 is what every flavor returns on
                // its happy-path health endpoint.
                .with_expected_status_code(200u16),
        )))
        // Boot defaults. extra_env overrides any of these.
        .with_env_var("EVAL_MODEL", "openai/azure/gpt-5.4")
        .with_env_var("OPENAI_API_KEY", "sk-bogus")
        .with_env_var("OPENAI_API_BASE", "http://127.0.0.1:9999/v1")
        .with_env_var("HOST", "0.0.0.0");

    for (k, v) in extra_env {
        img = img.with_env_var(*k, *v);
    }

    img.start().await.expect("start gateway")
}

async fn gateway_port(c: &ContainerAsync<GenericImage>) -> u16 {
    c.get_host_port_ipv4(ContainerPort::Tcp(4000))
        .await
        .expect("get host port")
}

fn http() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("build reqwest client")
}

fn body_openai() -> Value {
    json!({
        "model": "openai/azure/gpt-5.4",
        "messages": [{"role": "user", "content": "Reply with exactly: OK"}]
    })
}

fn body_anthropic() -> Value {
    json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 20,
        "messages": [{"role": "user", "content": "Reply with exactly: OK"}]
    })
}

fn body_genai() -> Value {
    json!({
        "contents": [{"role": "user", "parts": [{"text": "Reply with exactly: OK"}]}]
    })
}

fn upstream_creds() -> (String, String) {
    let _ = dotenvy::dotenv();
    let key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        panic!("OPENAI_API_KEY not set — required for #[ignore] tests. Populate .env or skip.")
    });
    let base = std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| {
        panic!("OPENAI_API_BASE not set — required for #[ignore] tests. Populate .env or skip.")
    });
    (key, base)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Static (no runtime, no creds) — Dockerfile text + filesystem checks.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn static_each_flavor_declares_gateway_kind_label() {
    for flavor in FLAVORS {
        let txt = dockerfile_text(flavor);
        assert!(
            txt.contains(&format!(r#"LABEL gateway.kind="{flavor}""#)),
            "gateways/{flavor}/Dockerfile missing LABEL gateway.kind=\"{flavor}\""
        );
    }
}

#[test]
fn static_each_flavor_declares_upstream_version_label() {
    // Every flavor must pin the upstream version in a `gateway.<kind>_version`
    // label so version drift is grep-able from the image metadata.
    let cases: &[(&str, &str)] = &[
        ("bifrost", "gateway.bifrost_version="),
        ("litellm", "gateway.litellm_version="),
        ("portkey", "gateway.portkey_version="),
    ];
    for (flavor, needle) in cases {
        assert!(
            dockerfile_text(flavor).contains(needle),
            "gateways/{flavor}/Dockerfile missing `LABEL {needle}...` — upstream version drift becomes invisible"
        );
    }
}

#[test]
fn static_bifrost_translates_protocols_label_is_true() {
    assert!(
        dockerfile_text("bifrost").contains(r#"LABEL gateway.translates_protocols="true""#),
        "bifrost natively serves /openai, /anthropic, /genai with cross-translation"
    );
}

#[test]
fn static_litellm_translates_protocols_label_is_true() {
    assert!(
        dockerfile_text("litellm").contains(r#"LABEL gateway.translates_protocols="true""#),
        "litellm natively serves all three protocols with cross-translation"
    );
}

#[test]
fn static_portkey_translates_protocols_label_is_false() {
    // Portkey self-hosted only does /openai; /anthropic + /genai return
    // 501. This label asserts that honest contract — was "true" when we
    // bundled a bifrost sidecar to fake coverage (rule 8: fail loud, do
    // not silently fall back to a translator that pretends to be portkey).
    assert!(
        dockerfile_text("portkey").contains(r#"LABEL gateway.translates_protocols="false""#),
        "portkey label must be `false` since the bifrost sidecar was removed"
    );
}

#[test]
fn static_portkey_declares_only_openai_protocol() {
    assert!(
        dockerfile_text("portkey").contains(r#"LABEL gateway.protocols="openai""#),
        "portkey must declare its actual protocol coverage in `gateway.protocols`"
    );
}

#[test]
fn static_portkey_has_no_bifrost_reference() {
    // If anyone re-adds a bifrost sidecar to fake protocol coverage,
    // this test trips. The 501 contract is the load-bearing invariant
    // (see Caddyfile + RULES.md rule 8).
    let txt = dockerfile_text("portkey");
    let forbidden = [
        "FROM docker.io/maximhq/bifrost",
        "FROM --platform=linux/amd64 docker.io/maximhq/bifrost",
        "/opt/gateway/bifrost",
        "bifrost-config.json",
        "bifrost/main",
    ];
    for needle in forbidden {
        assert!(
            !txt.contains(needle),
            "gateways/portkey/Dockerfile contains '{needle}' — portkey must not bundle bifrost (rule 8: \
             fail loud on unsupported protocols, do not silently fall back to a sidecar translator)"
        );
    }
}

#[test]
fn static_portkey_has_no_bundled_bifrost_config() {
    assert!(
        !test_support::repo_root()
            .join("containers/gateways/portkey/bifrost-config.json")
            .exists(),
        "gateways/portkey/bifrost-config.json must not exist — sidecar config was removed alongside the binary"
    );
}

#[test]
fn static_portkey_caddyfile_returns_501_on_anthropic_and_genai() {
    let cf = std::fs::read_to_string(
        test_support::repo_root().join("containers/gateways/portkey/Caddyfile"),
    )
    .expect("read gateways/portkey/Caddyfile");
    // The Caddyfile MUST short-circuit these protocols before any
    // upstream call. We assert on the response code + body marker.
    assert!(
        cf.contains("handle /anthropic/*") && cf.contains("501"),
        "portkey Caddyfile must return 501 on /anthropic — found no '501' near a /anthropic handler"
    );
    assert!(
        cf.contains("handle /genai/*") && cf.contains("501"),
        "portkey Caddyfile must return 501 on /genai — found no '501' near a /genai handler"
    );
    assert!(
        cf.contains("not_implemented"),
        "portkey Caddyfile must emit `not_implemented` in the 501 error body"
    );
}

#[test]
fn static_portkey_health_does_not_probe_removed_sidecar() {
    // Regression guard: when we stripped the bifrost sidecar, the
    // health probe still pointed at 127.0.0.1:4002. That makes the
    // gateway perpetually unhealthy with no obvious symptom — the kind
    // of bug a test should catch the next time someone refactors.
    let health = std::fs::read_to_string(
        test_support::repo_root().join("containers/gateways/portkey/health"),
    )
    .expect("read gateways/portkey/health");
    assert!(
        !health.contains(":4002"),
        "gateways/portkey/health still probes :4002 (bifrost sidecar port) — sidecar was removed, update the probe"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Boot (runtime, no creds) — the gateway listens on :4000 after start.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

async fn assert_boots(flavor: &str) {
    // The HttpWaitStrategy in start_gateway is the assertion: if it
    // returns, the per-flavor health endpoint is serving 2xx. The test
    // body exists so failures attribute to a specific flavor.
    let _c = start_gateway(flavor, &[]).await;
}

#[tokio::test]
async fn boot_bifrost() {
    assert_boots("bifrost").await
}

#[tokio::test]
async fn boot_litellm() {
    assert_boots("litellm").await
}

#[tokio::test]
async fn boot_portkey() {
    assert_boots("portkey").await
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Protocol matrix without creds — portkey 501s. Caddy short-circuits
// before any upstream call, so these tests don't need credentials.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn portkey_anthropic_returns_501_not_implemented() {
    let c = start_gateway("portkey", &[]).await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!("http://127.0.0.1:{port}/anthropic/v1/messages"))
        .json(&body_anthropic())
        .send()
        .await
        .expect("post anthropic");
    assert_eq!(
        resp.status(),
        501,
        "portkey /anthropic must return 501 (cf. RULES.md rule 8: no silent translator fallback)"
    );
    assert!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.starts_with("application/json"))
            .unwrap_or(false),
        "501 body must be served as application/json so strict clients parse without sniffing"
    );
    let body: Value = resp.json().await.expect("parse 501 body");
    assert_eq!(body["error"]["type"], "not_implemented");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("Anthropic"),
        "501 message must name the protocol — got: {msg:?}"
    );
    assert!(
        msg.contains("bifrost") || msg.contains("litellm"),
        "501 message must point at a working flavor — got: {msg:?}"
    );
}

#[tokio::test]
async fn portkey_genai_returns_501_not_implemented() {
    let c = start_gateway("portkey", &[]).await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/genai/v1beta/models/gemini-2.5-pro:generateContent"
        ))
        .json(&body_genai())
        .send()
        .await
        .expect("post genai");
    assert_eq!(resp.status(), 501);
    let body: Value = resp.json().await.expect("parse 501 body");
    assert_eq!(body["error"]["type"], "not_implemented");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("Gemini") || msg.contains("genai"),
        "501 message must name the protocol — got: {msg:?}"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Upstream-credentialed happy path (#[ignore]) — real chat completion
// against the upstream in .env, asserts protocol-shaped 200 response.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

async fn start_with_real_creds(flavor: &str) -> ContainerAsync<GenericImage> {
    let (key, base) = upstream_creds();
    start_gateway(
        flavor,
        &[("OPENAI_API_KEY", &key), ("OPENAI_API_BASE", &base)],
    )
    .await
}

async fn assert_openai_200(flavor: &str) {
    let c = start_with_real_creds(flavor).await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/openai/v1/chat/completions"
        ))
        .json(&body_openai())
        .send()
        .await
        .expect("post chat completions");
    let status = resp.status();
    let body: Value = resp.json().await.expect("parse 200 body");
    assert_eq!(status, 200, "{flavor} /openai → {status} body={body}");
    assert!(
        body["choices"][0]["message"]["content"].is_string(),
        "{flavor} /openai response missing choices[0].message.content: {body}"
    );
}

#[tokio::test]
#[ignore]
async fn upstream_bifrost_openai() {
    assert_openai_200("bifrost").await
}
#[tokio::test]
#[ignore]
async fn upstream_litellm_openai() {
    assert_openai_200("litellm").await
}
#[tokio::test]
#[ignore]
async fn upstream_portkey_openai() {
    assert_openai_200("portkey").await
}

async fn assert_anthropic_200(flavor: &str) {
    let c = start_with_real_creds(flavor).await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!("http://127.0.0.1:{port}/anthropic/v1/messages"))
        .json(&body_anthropic())
        .send()
        .await
        .expect("post anthropic");
    let status = resp.status();
    let body: Value = resp.json().await.expect("parse 200 body");
    assert_eq!(status, 200, "{flavor} /anthropic → {status} body={body}");
    assert!(
        body["content"].is_array(),
        "{flavor} /anthropic response missing content array: {body}"
    );
}

#[tokio::test]
#[ignore]
async fn upstream_bifrost_anthropic() {
    assert_anthropic_200("bifrost").await
}
#[tokio::test]
#[ignore]
async fn upstream_litellm_anthropic() {
    assert_anthropic_200("litellm").await
}

async fn assert_genai_200(flavor: &str) {
    let c = start_with_real_creds(flavor).await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/genai/v1beta/models/gemini-2.5-pro:generateContent"
        ))
        .json(&body_genai())
        .send()
        .await
        .expect("post genai");
    let status = resp.status();
    let body: Value = resp.json().await.expect("parse 200 body");
    assert_eq!(status, 200, "{flavor} /genai → {status} body={body}");
    assert!(
        body["candidates"].is_array(),
        "{flavor} /genai response missing candidates: {body}"
    );
}

#[tokio::test]
#[ignore]
async fn upstream_bifrost_genai() {
    assert_genai_200("bifrost").await
}
#[tokio::test]
#[ignore]
async fn upstream_litellm_genai() {
    assert_genai_200("litellm").await
}

// Streaming must keep routing: the bifrost config allow-lists the *_stream ops
// (to skip list_models without blocking inference). Assert 200 + SSE chunks.
async fn assert_openai_stream_200(flavor: &str) {
    let c = start_with_real_creds(flavor).await;
    let port = gateway_port(&c).await;
    let mut body = body_openai();
    body["stream"] = json!(true);
    body["max_tokens"] = json!(16);
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/openai/v1/chat/completions"
        ))
        .json(&body)
        .send()
        .await
        .expect("post streaming chat completions");
    let status = resp.status();
    let text = resp.text().await.expect("read SSE body");
    assert_eq!(
        status, 200,
        "{flavor} /openai stream → {status} body={text}"
    );
    assert!(
        text.lines().any(|l| l.starts_with("data:")),
        "{flavor} /openai stream returned no SSE data chunks: {text}"
    );
}

#[tokio::test]
#[ignore]
async fn upstream_bifrost_openai_stream() {
    assert_openai_stream_200("bifrost").await
}

#[tokio::test]
#[ignore]
async fn upstream_portkey_anthropic_remains_501() {
    // Structural invariant — credentials don't change protocol coverage.
    let c = start_with_real_creds("portkey").await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!("http://127.0.0.1:{port}/anthropic/v1/messages"))
        .json(&body_anthropic())
        .send()
        .await
        .expect("post anthropic");
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
#[ignore]
async fn upstream_portkey_genai_remains_501() {
    let c = start_with_real_creds("portkey").await;
    let port = gateway_port(&c).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/genai/v1beta/models/gemini-2.5-pro:generateContent"
        ))
        .json(&body_genai())
        .send()
        .await
        .expect("post genai");
    assert_eq!(resp.status(), 501);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// OTel emission (#[ignore]) — otelcol sidecar on a shared network + a
// bind-mounted /output. After a real chat completion, traces.jsonl
// must contain the gen_ai.* OTel semconv attributes.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const GEN_AI_REQUIRED_ATTRS: &[&str] = &[
    // Per the OTel gen_ai semconv spec — the minimum set every span on
    // a successful LLM call should carry.
    "gen_ai.input.messages",
    "gen_ai.output.messages",
    "gen_ai.response.model",
];

/// Start otelcol + gateway on a shared bridge network. `host_output`
/// is bind-mounted into otelcol so the test process can read the
/// emitted traces.jsonl directly.
async fn start_pod_with_otel(
    flavor: &str,
    host_output: &Path,
) -> (ContainerAsync<GenericImage>, ContainerAsync<GenericImage>) {
    ensure_built().await;
    let (key, base) = upstream_creds();

    // Unique network per pod so parallel test threads don't collide
    // when multiple OTel tests run side by side.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let net = format!("gw-test-{flavor}-{nanos}");

    // Order matters: `GenericImage` methods (with_wait_for) MUST come
    // before any `ImageExt` method (with_platform, with_mount, ...)
    // because the ImageExt calls convert to ContainerRequest, on which
    // with_wait_for is not defined.
    let otel = GenericImage::new("ghcr.io/exgentic/core/otel", "latest")
        .with_wait_for(WaitFor::message_on_stderr(
            "Everything is ready. Begin running and processing data.",
        ))
        .with_mount(Mount::bind_mount(
            host_output.to_str().expect("utf8 host path"),
            "/output",
        ))
        .with_network(&net)
        // container_name=otelcol-{nanos} so the gateway can reach it
        // as OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol-<nanos>:4318
        // — the bridge-network DNS resolves the container name.
        .with_container_name(format!("otelcol-{nanos}"))
        .start()
        .await
        .expect("start otelcol");

    let (name, tag) = gateway_image_ref(flavor);
    let gw = GenericImage::new(name, tag)
        .with_exposed_port(ContainerPort::Tcp(4000))
        .with_wait_for(WaitFor::Http(Box::new(
            HttpWaitStrategy::new(health_path(flavor))
                .with_port(ContainerPort::Tcp(4000))
                .with_poll_interval(Duration::from_millis(500))
                .with_expected_status_code(200u16),
        )))
        // Share /output with the otelcol sidecar so the litellm
        // eval_logger callback's writes to /output/trajectory.jsonl +
        // /output/result.json are visible to the test process. otelcol
        // also has the same mount, so traces.jsonl (its own output)
        // and trajectory.jsonl (the gateway's) co-exist in one dir.
        .with_mount(Mount::bind_mount(
            host_output.to_str().expect("utf8 host path"),
            "/output",
        ))
        .with_network(&net)
        .with_env_var("EVAL_MODEL", "openai/azure/gpt-5.4")
        .with_env_var("OPENAI_API_KEY", key)
        .with_env_var("OPENAI_API_BASE", base)
        .with_env_var("HOST", "0.0.0.0")
        // Both gateway flavors honor this — bifrost via its native
        // `otel` plugin (collector URL derived in /opt/gateway/start),
        // litellm via the `otel` callback in config.yaml.template.
        .with_env_var(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            format!("http://otelcol-{nanos}:4318"),
        )
        .start()
        .await
        .expect("start gateway");

    (otel, gw)
}

/// Wait for traces.jsonl to appear AND contain a `gen_ai.` span attr.
///
/// Polling until "non-empty" is not enough: litellm emits startup
/// telemetry spans (health checks, model discovery) that the otelcol
/// batch processor may flush before the chat-completion span lands.
/// Returning early on those stale spans makes the test report a false
/// "missing gen_ai.*" failure. Wait for the actual LLM-call span by
/// requiring `gen_ai.` to be present.
///
/// Up to 30s — covers OpenTelemetry SDK's 5s default
/// BatchSpanProcessor schedule_delay_millis + otelcol's 200ms batch
/// timeout + filesystem flush slack.
fn await_traces_with_gen_ai(host_output: &Path) -> String {
    let path = host_output.join("traces.jsonl");
    let mut last = String::new();
    for _ in 0..60 {
        if let Ok(s) = std::fs::read_to_string(&path) {
            last = s;
            if last.contains("gen_ai.") {
                return last;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    panic!(
        "traces.jsonl at {} never carried a gen_ai.* attribute within 30s. \
         Latest content ({} bytes): {}",
        path.display(),
        last.len(),
        &last[..last.len().min(800)]
    );
}

async fn assert_gen_ai_attrs(flavor: &str) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_otel, gw) = start_pod_with_otel(flavor, tmp.path()).await;
    let port = gateway_port(&gw).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/openai/v1/chat/completions"
        ))
        .json(&body_openai())
        .send()
        .await
        .expect("post chat");
    assert_eq!(resp.status(), 200, "precondition: gateway returned 200");

    let traces = await_traces_with_gen_ai(tmp.path());
    for attr in GEN_AI_REQUIRED_ATTRS {
        assert!(
            traces.contains(attr),
            "{flavor} traces.jsonl missing OTel gen_ai semconv attribute `{attr}` — \
             gateways MUST emit the canonical gen_ai.* span attributes. \
             First 600 bytes of traces.jsonl: {}",
            &traces[..traces.len().min(600)]
        );
    }
}

#[tokio::test]
#[ignore]
async fn otel_bifrost_gen_ai_attrs() {
    assert_gen_ai_attrs("bifrost").await
}

#[tokio::test]
#[ignore]
async fn otel_litellm_gen_ai_attrs() {
    assert_gen_ai_attrs("litellm").await
}

#[tokio::test]
#[ignore]
async fn otel_litellm_writes_trajectory_jsonl_and_result_json() {
    // litellm has a custom callback (eval_logger.eval_logger_instance)
    // in addition to the OTel one. This is what models/replay consumes
    // — if it stops firing, every replay fixture goes stale silently.
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_otel, gw) = start_pod_with_otel("litellm", tmp.path()).await;
    let port = gateway_port(&gw).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/openai/v1/chat/completions"
        ))
        .json(&body_openai())
        .send()
        .await
        .expect("post chat");
    assert_eq!(resp.status(), 200);

    // The callback writes synchronously after each request, but the
    // test process and the container don't share a fsync — give the
    // file a beat to appear.
    let trajectory = tmp.path().join("trajectory.jsonl");
    let result = tmp.path().join("result.json");
    for _ in 0..20 {
        if trajectory.exists() && result.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(
        trajectory.exists(),
        "litellm did not write /output/trajectory.jsonl — eval_logger callback not firing"
    );
    assert!(
        result.exists(),
        "litellm did not write /output/result.json — eval_logger callback not firing"
    );
    // trajectory entries must parse and carry at least `response`.
    let line1 = std::fs::read_to_string(&trajectory)
        .unwrap()
        .lines()
        .next()
        .expect("trajectory.jsonl has no lines")
        .to_string();
    let entry: Value = serde_json::from_str(&line1).expect("trajectory line must be JSON");
    assert!(
        entry.get("response").is_some(),
        "trajectory entry missing `response` field — StandardLoggingPayload shape changed?"
    );
}

#[tokio::test]
#[ignore]
async fn otel_portkey_openai_emits_no_gateway_spans() {
    // Documents the known portkey limitation: portkey self-hosted
    // v1.15.x has no @opentelemetry deps in its Node bundle, so
    // gateway-side spans for /openai are absent. The runner's
    // OTEL_EXPORTER_OTLP_ENDPOINT covers agents that emit OTel
    // client-side, but the gateway itself does not. If portkey ever
    // ships OTel instrumentation, this test trips — at that point
    // delete the test, drop the comment in gateways/portkey/start,
    // and add an `assert_gen_ai_attrs("portkey")` instead.
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_otel, gw) = start_pod_with_otel("portkey", tmp.path()).await;
    let port = gateway_port(&gw).await;
    let resp = http()
        .post(format!(
            "http://127.0.0.1:{port}/openai/v1/chat/completions"
        ))
        .json(&body_openai())
        .send()
        .await
        .expect("post chat");
    assert_eq!(resp.status(), 200);
    std::thread::sleep(Duration::from_secs(3));

    let traces = tmp.path().join("traces.jsonl");
    let has_gen_ai = traces.exists()
        && std::fs::read_to_string(&traces)
            .unwrap_or_default()
            .contains("gen_ai.input.messages");
    assert!(
        !has_gen_ai,
        "portkey emitted gateway-side gen_ai spans for /openai — is portkey now shipping OTel? \
         Update the comment in gateways/portkey/start and switch this test to assert presence."
    );
}
