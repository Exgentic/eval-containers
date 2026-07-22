//! Gateway translation contract — client request → forwarded ("target") request.
//!
//! A recording mock upstream captures exactly what each gateway forwards; every
//! case asserts the forwarded wire (path), the pinned model, and the search tool.
//! No credentials, no response assertions.
//!
//! `EVAL_MODEL` is the model authority — pinned whenever set. `EVAL_MODEL_API`
//! only overrides the wire:
//!   - native_pin  (EVAL_MODEL set, EVAL_MODEL_API unset) — the production
//!     default: pin the model, keep the inbound wire, tools survive.
//!   - translate_* (EVAL_MODEL_API set) — force that wire; matched inbound keeps
//!     its tool, cross-protocol inbound is `KnownLossy`.
//!   - passthrough (EVAL_MODEL unset) — client model forwarded unchanged.
//!
//! 10 tests, one gateway boot per (flavor, config). Every-PR smoke = the 2
//! native_pin groups; the rest are `#[ignore]`-gated (release, `--ignored`).
//!
//! ## Run
//!   cargo test --test translation      # builds images on first run; needs DOCKER_HOST

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

// ─── Build bootstrap (translating flavors only) ──────────────────────

static IMAGES_BUILT: OnceCell<()> = OnceCell::const_new();

async fn ensure_built() {
    IMAGES_BUILT
        .get_or_init(|| async {
            let _ = dotenvy::dotenv();
            common::bake_targets(&[
                "gateway-bifrost",
                "model-bifrost",
                "gateway-litellm",
                "model-litellm",
            ])
            .await;
        })
        .await;
}

fn health_path(flavor: &str) -> &'static str {
    match flavor {
        "bifrost" => "/api/health",
        "litellm" => "/health/liveness",
        _ => panic!("unknown flavor: {flavor}"),
    }
}

fn nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn http() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("build reqwest client")
}

/// Start the recording mock upstream + the gateway on a shared network.
/// `host_output` is bind-mounted into the mock so the test reads the
/// captured requests directly. The gateway's upstream env vars point at
/// the mock; OTel is pointed there too (harmless — trace POSTs land on
/// /v1/traces and are filtered out of the assertion).
async fn start_pod(
    flavor: &str,
    eval_model_api: &str,
    eval_model: &str,
    host_output: &Path,
) -> (ContainerAsync<GenericImage>, ContainerAsync<GenericImage>) {
    ensure_built().await;
    let n = nanos();
    let net = format!("xl8-{flavor}-{n}");
    let mock_name = format!("mock-{n}");
    let mock_script = test_support::repo_root().join("tests/run/gateways/mock_upstream.py");

    let mock = GenericImage::new("python", "3.12-slim")
        .with_exposed_port(ContainerPort::Tcp(8080))
        .with_wait_for(WaitFor::Http(Box::new(
            HttpWaitStrategy::new("/")
                .with_port(ContainerPort::Tcp(8080))
                .with_poll_interval(Duration::from_millis(200))
                .with_expected_status_code(200u16),
        )))
        .with_mount(Mount::bind_mount(
            mock_script.to_str().expect("utf8 script path"),
            "/mock_upstream.py",
        ))
        .with_mount(Mount::bind_mount(
            host_output.to_str().expect("utf8 host path"),
            "/output",
        ))
        .with_network(&net)
        .with_container_name(&mock_name)
        .with_cmd(["python", "/mock_upstream.py"])
        .start()
        .await
        .expect("start mock upstream");

    let upstream = format!("http://{mock_name}:8080");
    let gw = GenericImage::new(
        format!("ghcr.io/exgentic/models/{flavor}"),
        "latest".to_string(),
    )
    .with_exposed_port(ContainerPort::Tcp(4000))
    .with_wait_for(WaitFor::Http(Box::new(
        HttpWaitStrategy::new(health_path(flavor))
            .with_port(ContainerPort::Tcp(4000))
            .with_poll_interval(Duration::from_millis(500))
            .with_expected_status_code(200u16),
    )))
    .with_network(&net)
    // No HOST var: the gateway must serve on :4000 out of the box. An empty
    // eval_model_api is the native-pin default (or passthrough if eval_model is
    // also empty); a value forces that wire.
    .with_env_var("EVAL_MODEL_API", eval_model_api)
    .with_env_var("EVAL_MODEL", eval_model)
    .with_env_var("OPENAI_API_KEY", "sk-mock")
    .with_env_var("OPENAI_API_BASE", &upstream)
    .with_env_var("OTEL_EXPORTER_OTLP_ENDPOINT", &upstream)
    .start()
    .await
    .expect("start gateway");

    (mock, gw)
}

async fn gateway_port(c: &ContainerAsync<GenericImage>) -> u16 {
    c.get_host_port_ipv4(ContainerPort::Tcp(4000))
        .await
        .expect("get host port")
}

/// All upstream-facing inference requests the gateway has forwarded so far, in
/// order. Reads `requests.jsonl` (bind-mounted) and keeps entries whose path is
/// an LLM inference endpoint (ignoring OTel /v1/traces + discovery GETs). A
/// group fires requests sequentially and watches this list grow by one.
fn all_inference_requests(reqs_file: &Path) -> Vec<Value> {
    let is_inference = |p: &str| {
        (p.contains("messages") && !p.contains("count_tokens"))
            || p.contains("chat/completions")
            || p.contains("responses")
            || p.contains("generateContent")
    };
    std::fs::read_to_string(reqs_file)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|e| e["path"].as_str().map(is_inference).unwrap_or(false))
        .collect()
}

/// The three demands a forwarded request must satisfy.
struct Target {
    /// Substring the forwarded path MUST contain (native protocol marker):
    /// the gateway forwarded on the API's own native path, not a translated one.
    path_marker: &'static str,
    /// The pinned `EVAL_MODEL` handle the forwarded request MUST reference
    /// (in body.model or the URL path) — proves the gateway REWROTE the
    /// client's model to the target. The client always sends a different model.
    model: &'static str,
    /// What must happen to the search tool on the forward.
    tool: ToolExpect,
}

/// Expectation for the native search tool after the forward.
enum ToolExpect {
    /// No tool in this request.
    None,
    /// Tool MUST survive (matched-protocol search — no translation).
    Require(ToolCheck),
    /// Cross-protocol translation: the server tool is KNOWN to not survive
    /// losslessly. We assert routing + pin only, and merely NOTE if a future
    /// engine version unexpectedly preserves it (then tighten to Require).
    KnownLossy(ToolCheck),
}

#[derive(Clone, Copy)]
enum ToolCheck {
    /// tools[].type == "web_search_20250305"
    AnthropicWebSearch,
    /// tools[].type == "web_search_preview"
    OpenAiWebSearch,
    /// tools[].google_search present
    GoogleSearch,
}

fn tool_survives(body: &Value, check: ToolCheck) -> bool {
    let tools = body.get("tools").and_then(Value::as_array);
    let Some(tools) = tools else { return false };
    tools.iter().any(|t| match check {
        // bifrost may re-stamp the tool to a newer dated version
        // (web_search_20250305 -> web_search_20260209); accept any dated
        // web_search_<date> (but not the OpenAI "web_search_preview").
        ToolCheck::AnthropicWebSearch => t["type"]
            .as_str()
            .is_some_and(|s| s.starts_with("web_search_2")),
        ToolCheck::OpenAiWebSearch => t["type"] == "web_search_preview",
        // Accept either spelling: clients may send snake_case `google_search`,
        // and the native Gemini REST field is camelCase `googleSearch` — the
        // tool surviving in either form satisfies the contract.
        ToolCheck::GoogleSearch => {
            t.get("google_search").is_some() || t.get("googleSearch").is_some()
        }
    })
}

/// The forwarded request references the pinned model (body.model or URL path).
fn references_model(body: &Value, fwd_path: &str, model: &str) -> bool {
    body["model"].as_str() == Some(model) || fwd_path.contains(model)
}

/// Assert one forwarded request against its target: target-wire protocol, model
/// pinned to `EVAL_MODEL`, and the search-tool expectation (pure — no I/O).
fn assert_request(flavor: &str, inbound_path: &str, req: &Value, target: &Target) {
    let fwd_path = req["path"].as_str().unwrap_or("");
    let body = &req["body"];

    // Demand 1 — target-wire protocol. path_marker may list alternatives
    // ("responses|completions"); any match satisfies (both are OpenAI-native).
    assert!(
        target.path_marker.split('|').any(|m| fwd_path.contains(m)),
        "{flavor} {inbound_path}: forwarded to `{fwd_path}` but expected a `{}` \
         (target-wire) path — routed to the wrong protocol. body={body}",
        target.path_marker
    );
    // Demand 2 — model rewritten to the pinned EVAL_MODEL target.
    assert!(
        references_model(body, fwd_path, target.model),
        "{flavor} {inbound_path}: did not pin the model to `{}` — the gateway must \
         rewrite the client's model. forwarded model={:?} path={fwd_path}",
        target.model,
        body["model"]
    );
    // Demand 3 — native search tool.
    match &target.tool {
        ToolExpect::None => {}
        ToolExpect::Require(check) => assert!(
            tool_survives(body, *check),
            "{flavor} {inbound_path}: dropped/mangled its native search tool. body={body}"
        ),
        ToolExpect::KnownLossy(check) => {
            if tool_survives(body, *check) {
                eprintln!(
                    "NOTE {flavor} {inbound_path}: cross-protocol search tool UNEXPECTEDLY \
                     survived translation — an engine may have fixed it; tighten to Require."
                );
            }
        }
    }
}

/// One input->target assertion. Many share a single running gateway.
struct Case {
    inbound_path: &'static str,
    input: Value,
    target: Target,
}

fn case(
    inbound_path: &'static str,
    input: Value,
    path_marker: &'static str,
    model: &'static str,
    tool: ToolExpect,
) -> Case {
    Case {
        inbound_path,
        input,
        target: Target {
            path_marker,
            model,
            tool,
        },
    }
}

/// Boot ONE gateway in a fixed mode (translate target, or passthrough) and fire
/// every case's request at it in sequence. The mode is set once at container
/// boot via env; only the INBOUND protocol varies per request, so every case
/// sharing a target config reuses a single container (28 assertions, 8 boots).
async fn run_group(flavor: &str, eval_model_api: &str, eval_model: &str, cases: Vec<Case>) {
    let tmp = tempfile::tempdir().expect("tempdir");
    // EVAL_MODEL_API set -> translate to that wire + pin EVAL_MODEL; unset ->
    // passthrough (client wire + client model). EVAL_MODEL is a bare handle.
    let (_mock, gw) = start_pod(flavor, eval_model_api, eval_model, tmp.path()).await;
    let port = gateway_port(&gw).await;
    let reqs_file = tmp.path().join("requests.jsonl");

    let mut seen = 0usize;
    for c in &cases {
        // Fire-and-tolerate: the gateway may 4xx/5xx on the mock's canned
        // response — we only care what it FORWARDED, which the mock recorded.
        let _ = http()
            .post(format!("http://127.0.0.1:{port}{}", c.inbound_path))
            .json(&c.input)
            .send()
            .await;

        // Wait for THIS request's forward to land (a new inference line) so a
        // slow bind-mount flush can't hand us the previous case's forward.
        let mut got = None;
        for _ in 0..40 {
            let reqs = all_inference_requests(&reqs_file);
            if reqs.len() > seen {
                seen = reqs.len();
                got = reqs.into_iter().last();
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let req = got.unwrap_or_else(|| {
            let dump = std::fs::read_to_string(&reqs_file).unwrap_or_default();
            panic!(
                "{flavor} {}: no new inference request within 8s. requests.jsonl ({} bytes): {}",
                c.inbound_path,
                dump.len(),
                &dump[..dump.len().min(1200)]
            )
        });
        assert_request(flavor, c.inbound_path, &req, &c.target);
    }
}

// ─── Pinned targets (EVAL_MODEL — bare handles, provisioned upstream) ─
// The provider is chosen by the inbound address, so no `<provider>/` prefix.
const ANTHROPIC_MODEL: &str = "aws/claude-opus-4-8";
const OPENAI_MODEL: &str = "azure/gpt-5.4";
const GOOGLE_MODEL: &str = "gcp/gemini-3-flash-preview";

// Target wire APIs (EVAL_MODEL_API) — the matched cells set the API equal to
// the inbound protocol (so it's a native passthrough that still pins the model).
const API_ANTHROPIC: &str = "anthropic";
const API_OPENAI: &str = "openai";
const API_GEMINI: &str = "gemini";

// Neutral pinned handle for the native-pin default (no wire prefix — the wire is
// the inbound protocol). Distinct from the per-wire handles so the assertion
// unambiguously shows the gateway rewrote the client model to EVAL_MODEL.
const PIN_MODEL: &str = "vendor/pinned-xyz";

// ─── Input bodies — the client sends a DIFFERENT model than the target,
//     so a passing model assertion proves the gateway rewrote it. ──────
fn anthropic_chat() -> Value {
    json!({"model":"claude-sonnet-4-5","max_tokens":32,
        "messages":[{"role":"user","content":"hi"}]})
}
fn anthropic_search() -> Value {
    json!({"model":"claude-sonnet-4-5","max_tokens":256,
        "messages":[{"role":"user","content":"search the web for today's date"}],
        "tools":[{"type":"web_search_20250305","name":"web_search"}]})
}
fn openai_chat() -> Value {
    json!({"model":"gpt-4o","input":"hi"})
}
fn openai_search() -> Value {
    json!({"model":"gpt-4o","input":"search the web for today's date",
        "tools":[{"type":"web_search_preview"}]})
}
fn google_chat() -> Value {
    json!({"contents":[{"role":"user","parts":[{"text":"hi"}]}]})
}
fn google_search() -> Value {
    json!({"contents":[{"role":"user","parts":[{"text":"search the web for today's date"}]}],
        "tools":[{"google_search":{}}]})
}

// The client's URL carries its own (different) model; the gateway must
// rewrite it to GOOGLE_MODEL.
const ANTHROPIC_PATH: &str = "/anthropic/v1/messages";
const OPENAI_PATH: &str = "/openai/v1/responses";
const GOOGLE_PATH: &str = "/genai/v1beta/models/gemini-2.5-pro:generateContent";

// One gateway boot per (flavor, config); each group fires every inbound protocol
// at the same container. Modes + buckets are described in the module doc above.

// NATIVE PIN (production default) — the passthrough-no-pin regression guard,
// so it's the only every-PR smoke: pin + keep the native wire + tools survive.
async fn native_pin_group(flavor: &str) {
    run_group(
        flavor,
        "", // EVAL_MODEL_API unset -> native wire, model still pinned
        PIN_MODEL,
        vec![
            case(
                ANTHROPIC_PATH,
                anthropic_chat(),
                "messages",
                PIN_MODEL,
                ToolExpect::None,
            ),
            case(
                ANTHROPIC_PATH,
                anthropic_search(),
                "messages",
                PIN_MODEL,
                ToolExpect::Require(ToolCheck::AnthropicWebSearch),
            ),
            case(
                OPENAI_PATH,
                openai_search(),
                "responses|completions",
                PIN_MODEL,
                ToolExpect::Require(ToolCheck::OpenAiWebSearch),
            ),
            case(
                GOOGLE_PATH,
                google_search(),
                "generateContent",
                PIN_MODEL,
                ToolExpect::Require(ToolCheck::GoogleSearch),
            ),
        ],
    )
    .await
}
#[tokio::test]
async fn bifrost_native_pin() {
    native_pin_group("bifrost").await
}
#[tokio::test]
async fn litellm_native_pin() {
    native_pin_group("litellm").await
}

// translate -> Anthropic /v1/messages.
async fn anthropic_group(flavor: &str) {
    run_group(
        flavor,
        API_ANTHROPIC,
        ANTHROPIC_MODEL,
        vec![
            case(
                ANTHROPIC_PATH,
                anthropic_chat(),
                "messages",
                ANTHROPIC_MODEL,
                ToolExpect::None,
            ),
            case(
                ANTHROPIC_PATH,
                anthropic_search(),
                "messages",
                ANTHROPIC_MODEL,
                ToolExpect::Require(ToolCheck::AnthropicWebSearch),
            ),
            case(
                OPENAI_PATH,
                openai_search(),
                "messages",
                ANTHROPIC_MODEL,
                ToolExpect::KnownLossy(ToolCheck::OpenAiWebSearch),
            ),
            case(
                GOOGLE_PATH,
                google_search(),
                "messages",
                ANTHROPIC_MODEL,
                ToolExpect::KnownLossy(ToolCheck::GoogleSearch),
            ),
        ],
    )
    .await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn bifrost_translate_anthropic() {
    anthropic_group("bifrost").await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn litellm_translate_anthropic() {
    anthropic_group("litellm").await
}

// translate -> OpenAI /v1/responses (or /chat/completions; both OpenAI-native).
async fn openai_group(flavor: &str) {
    run_group(
        flavor,
        API_OPENAI,
        OPENAI_MODEL,
        vec![
            case(
                OPENAI_PATH,
                openai_chat(),
                "responses|completions",
                OPENAI_MODEL,
                ToolExpect::None,
            ),
            case(
                OPENAI_PATH,
                openai_search(),
                "responses|completions",
                OPENAI_MODEL,
                ToolExpect::Require(ToolCheck::OpenAiWebSearch),
            ),
            case(
                ANTHROPIC_PATH,
                anthropic_chat(),
                "responses|completions",
                OPENAI_MODEL,
                ToolExpect::None,
            ),
            case(
                ANTHROPIC_PATH,
                anthropic_search(),
                "responses|completions",
                OPENAI_MODEL,
                ToolExpect::KnownLossy(ToolCheck::AnthropicWebSearch),
            ),
            case(
                GOOGLE_PATH,
                google_search(),
                "responses|completions",
                OPENAI_MODEL,
                ToolExpect::KnownLossy(ToolCheck::GoogleSearch),
            ),
        ],
    )
    .await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn bifrost_translate_openai() {
    openai_group("bifrost").await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn litellm_translate_openai() {
    openai_group("litellm").await
}

// translate -> Gemini ...:generateContent.
async fn gemini_group(flavor: &str) {
    run_group(
        flavor,
        API_GEMINI,
        GOOGLE_MODEL,
        vec![
            case(
                GOOGLE_PATH,
                google_chat(),
                "generateContent",
                GOOGLE_MODEL,
                ToolExpect::None,
            ),
            case(
                GOOGLE_PATH,
                google_search(),
                "generateContent",
                GOOGLE_MODEL,
                ToolExpect::Require(ToolCheck::GoogleSearch),
            ),
            case(
                ANTHROPIC_PATH,
                anthropic_search(),
                "generateContent",
                GOOGLE_MODEL,
                ToolExpect::KnownLossy(ToolCheck::AnthropicWebSearch),
            ),
            case(
                OPENAI_PATH,
                openai_search(),
                "generateContent",
                GOOGLE_MODEL,
                ToolExpect::KnownLossy(ToolCheck::OpenAiWebSearch),
            ),
        ],
    )
    .await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn bifrost_translate_gemini() {
    gemini_group("bifrost").await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn litellm_translate_gemini() {
    gemini_group("litellm").await
}

// passthrough -> EVAL_MODEL_API unset: client model forwarded unchanged.
async fn passthrough_group(flavor: &str) {
    run_group(
        flavor,
        "",
        "",
        vec![case(
            ANTHROPIC_PATH,
            anthropic_chat(),
            "messages",
            "claude-sonnet-4-5",
            ToolExpect::None,
        )],
    )
    .await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn bifrost_passthrough() {
    passthrough_group("bifrost").await
}
#[tokio::test]
#[ignore = "release-only (run with --ignored)"]
async fn litellm_passthrough() {
    passthrough_group("litellm").await
}
