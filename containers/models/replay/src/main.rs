//! Replay model server — serves recorded LLM responses for deterministic tests.
//!
//! Reads an OpenTelemetry trace file (OTLP/JSON, one ExportTraceServiceRequest
//! per line — what an otelcol `file` exporter writes) and serves the recorded
//! gen_ai responses in order. Indistinguishable to the eval container from a
//! real proxy (models/RULES.md rule 17); needs no API keys.
//!
//! Both gateways encode `gen_ai.output.messages` as a JSON string, differently:
//!
//! ```text
//! litellm (OTEL GenAI 1.38): [{role, parts:[{type:"text",content}], tool_calls, finish_reason}]
//! bifrost (plain OpenAI):    [{role, content}]  (on two spans/call; the summary
//!                            HTTP span carries raw text and is dropped)
//! ```
//!
//! A turn is a span whose output.messages parses to a JSON array; turns are
//! ordered by startTimeUnixNano and deduped by gen_ai.response.id.
//!
//! Routes re-emit the recorded turn in whatever wire format the caller wants:
//!
//! ```text
//! POST /v1/chat/completions,  /openai/v1/chat/completions   -> OpenAI Chat Completions
//! POST /v1/responses,         /openai/v1/responses          -> OpenAI Responses
//! POST /v1/messages,          /anthropic/v1/messages        -> Anthropic Messages
//! POST /v1beta/models/...,    /genai/v1beta/models/...      -> Google Gemini
//! GET  /health, GET|HEAD /                                  -> ok
//! ```
//!
//! Env: REPLAY_TRACES (default /data/traces.jsonl), PORT (default 4000).
//! `replay-server health` probes localhost:PORT/health and exits 0/1 — the
//! distroless image has no shell or curl for a HEALTHCHECK.

use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// One recorded LLM turn, normalized from a gen_ai span.
#[derive(Clone)]
struct Turn {
    text: String,
    tool_calls: Vec<Value>,
    finish_reason: String,
    model: String,
}

static CALL_INDEX: AtomicUsize = AtomicUsize::new(0);
static ID_SEQ: AtomicUsize = AtomicUsize::new(0);

fn main() {
    if is_health_invocation() {
        std::process::exit(health_probe());
    }
    let port = env_port();
    let path = std::env::var("REPLAY_TRACES").unwrap_or_else(|_| "/data/traces.jsonl".into());
    let turns = load_turns(&path);
    eprintln!("[replay] loaded {} gen_ai turns from {path}", turns.len());
    serve(port, turns);
}

/// True when invoked as a health probe, two ways: the explicit `server health`
/// form, or invoked *as* `/opt/gateway/health` (argv[0] basename `health`). The
/// latter is the gateway-contract drop-in: services.yaml's default healthcheck
/// `["CMD", "/opt/gateway/health"]` runs that path with no args, so making the
/// binary answer to it lets replay occupy the gateway slot with no healthcheck
/// override — the same contract bifrost's shell `health` script honors.
fn is_health_invocation() -> bool {
    let mut args = std::env::args();
    let arg0 = args.next().unwrap_or_default();
    is_health_args(&arg0, args.next().as_deref())
}

/// Pure core of `is_health_invocation`, split out for unit tests.
fn is_health_args(arg0: &str, arg1: Option<&str>) -> bool {
    arg1 == Some("health")
        || std::path::Path::new(arg0)
            .file_name()
            .and_then(|f| f.to_str())
            == Some("health")
}

fn env_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4000)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A present-but-arbitrary id. Replay doesn't need real ids, only that the
/// field exists and looks plausible; uniqueness avoids accidental collisions.
fn gen_id(prefix: &str) -> String {
    let n = ID_SEQ.fetch_add(1, Ordering::Relaxed) as u64;
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0) as u64;
    format!("{prefix}{t:016x}{n:08x}")
}

// ── OTLP parsing ────────────────────────────────────────────────────

/// Flatten a span's `attributes: [{key, value:{<type>Value: ...}}]` into a map.
/// intValue arrives as a string-encoded int64.
fn flatten_attrs(span: &Value) -> Map<String, Value> {
    let mut m = Map::new();
    let Some(arr) = span.get("attributes").and_then(Value::as_array) else {
        return m;
    };
    for a in arr {
        let Some(k) = a.get("key").and_then(Value::as_str) else {
            continue;
        };
        let Some(v) = a.get("value") else { continue };
        if let Some(s) = v.get("stringValue").and_then(Value::as_str) {
            m.insert(k.into(), Value::String(s.into()));
        } else if let Some(iv) = v.get("intValue") {
            if let Some(n) = iv
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| iv.as_i64())
            {
                m.insert(k.into(), json!(n));
            }
        } else if let Some(d) = v.get("doubleValue").and_then(Value::as_f64) {
            m.insert(k.into(), json!(d));
        } else if let Some(b) = v.get("boolValue").and_then(Value::as_bool) {
            m.insert(k.into(), Value::Bool(b));
        }
    }
    m
}

/// `gen_ai.output.messages` parsed to its array, or [] when it isn't one — that
/// drops summary/HTTP spans that store the raw response text there (bifrost).
fn parse_output_messages(attrs: &Map<String, Value>) -> Vec<Value> {
    match attrs.get("gen_ai.output.messages") {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::String(s)) if !s.is_empty() => serde_json::from_str::<Value>(s)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn canonicalize(attrs: &Map<String, Value>) -> Turn {
    let mut text = String::new();
    let mut tool_calls: Vec<Value> = Vec::new();
    let mut finish_reason = String::new();

    for msg in parse_output_messages(attrs) {
        let Some(msg) = msg.as_object() else { continue };
        // litellm OTEL-1.38 `parts`
        if let Some(parts) = msg.get("parts").and_then(Value::as_array) {
            for p in parts {
                if p.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(t) = p.get("content").and_then(Value::as_str) {
                        text.push_str(t);
                    }
                }
            }
        }
        // bifrost / OpenAI plain `content` (string, or a list of content parts)
        match msg.get("content") {
            Some(Value::String(c)) => text.push_str(c),
            Some(Value::Array(parts)) => {
                for p in parts {
                    if let Some(t) = p
                        .get("text")
                        .and_then(Value::as_str)
                        .or_else(|| p.get("content").and_then(Value::as_str))
                    {
                        text.push_str(t);
                    }
                }
            }
            _ => {}
        }
        if let Some(tcs) = msg.get("tool_calls").and_then(Value::as_array) {
            tool_calls.extend(tcs.iter().cloned());
        }
        if let Some(fr) = msg.get("finish_reason").and_then(Value::as_str) {
            finish_reason = fr.into();
        }
    }

    // Some gateways (bifrost) put the finish reason on the span, not the message.
    if finish_reason.is_empty() {
        if let Some(fr) = attrs
            .get("gen_ai.response.finish_reason")
            .or_else(|| attrs.get("gen_ai.response.finish_reasons"))
            .and_then(Value::as_str)
        {
            // finish_reasons may be a JSON array string like ["stop"].
            finish_reason = serde_json::from_str::<Value>(fr)
                .ok()
                .and_then(|v| {
                    v.as_array()
                        .and_then(|a| a.first()?.as_str().map(String::from))
                })
                .unwrap_or_else(|| fr.into());
        }
    }
    if finish_reason.is_empty() {
        finish_reason = "stop".into();
    }

    let model = attrs
        .get("gen_ai.response.model")
        .and_then(Value::as_str)
        .or_else(|| attrs.get("gen_ai.request.model").and_then(Value::as_str))
        .unwrap_or("replay")
        .to_string();

    Turn {
        text,
        tool_calls,
        finish_reason,
        model,
    }
}

fn span_is_error(span: &Value) -> bool {
    let code = span.get("status").and_then(|s| s.get("code"));
    code.and_then(Value::as_i64) == Some(2)
        || code.and_then(Value::as_str) == Some("STATUS_CODE_ERROR")
}

fn span_start_ns(span: &Value) -> u128 {
    span.get("startTimeUnixNano")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<u128>().ok())
                .or_else(|| v.as_u64().map(u128::from))
        })
        .unwrap_or(0)
}

/// Parse an OTLP/JSON trace file into ordered, deduped canonical turns.
fn load_turns(path: &str) -> Vec<Turn> {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("[replay] WARNING: no trace file at {path}");
            return Vec::new();
        }
    };
    parse_turns(&raw)
}

/// Split out from `load_turns` so it's unit-testable without a file.
fn parse_turns(raw: &str) -> Vec<Turn> {
    // (start_ns, response_id, turn)
    let mut collected: Vec<(u128, String, Turn)> = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        for rs in req
            .get("resourceSpans")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for ss in rs
                .get("scopeSpans")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                for span in ss
                    .get("spans")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let attrs = flatten_attrs(span);
                    if parse_output_messages(&attrs).is_empty() || span_is_error(span) {
                        continue;
                    }
                    let rid = attrs
                        .get("gen_ai.response.id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    collected.push((span_start_ns(span), rid, canonicalize(&attrs)));
                }
            }
        }
    }
    collected.sort_by_key(|t| t.0);
    // Dedup by response id: one LLM call can surface on several spans (bifrost
    // emits an HTTP span + an `llm.call` span). Spans without an id are kept.
    let mut seen: HashSet<String> = HashSet::new();
    collected
        .into_iter()
        .filter(|(_, rid, _)| rid.is_empty() || seen.insert(rid.clone()))
        .map(|(_, _, turn)| turn)
        .collect()
}

// ── emitters: canonical turn -> provider wire format ────────────────

fn parse_args(arguments: Option<&Value>) -> Value {
    match arguments {
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or_else(|_| json!({ "_raw": s })),
        Some(other) => other.clone(),
        None => json!({}),
    }
}

/// A recorded tool call's `(id, name, raw_arguments)`, with defaults: a generated
/// `prefix`-id when unrecorded, `""` name, `"{}"` arguments. Every emitter pulls
/// the same three fields; `parse_args(Some(&args))` turns the raw arguments into a
/// parsed object where a provider wants one (Anthropic/Gemini input).
fn tool_parts(tc: &Value, prefix: &str) -> (Value, Value, Value) {
    let func = tc.get("function");
    let id = tc
        .get("id")
        .cloned()
        .unwrap_or_else(|| json!(gen_id(prefix)));
    let name = func
        .and_then(|f| f.get("name"))
        .cloned()
        .unwrap_or_else(|| json!(""));
    let args = func
        .and_then(|f| f.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!("{}"));
    (id, name, args)
}

fn emit_chat(t: &Turn) -> Value {
    let mut msg = json!({
        "role": "assistant",
        "content": if t.text.is_empty() { Value::Null } else { json!(t.text) },
    });
    if !t.tool_calls.is_empty() {
        msg["tool_calls"] = json!(t.tool_calls);
    }
    json!({
        "id": gen_id("chatcmpl-"),
        "object": "chat.completion",
        "created": now_secs(),
        "model": t.model,
        "choices": [{ "index": 0, "message": msg, "finish_reason": t.finish_reason }],
        "usage": usage_openai(),
    })
}

/// A zero `usage` block (OpenAI shape). Replay doesn't track tokens, but a real
/// gateway (bifrost) maps this onto the client's provider usage — and a client
/// like claude-code reads `usage.input_tokens` and crashes if it's absent. So
/// the field must always be present, even as zeros.
fn usage_openai() -> Value {
    json!({ "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 })
}

fn emit_responses(t: &Turn) -> Value {
    let mut output: Vec<Value> = Vec::new();
    if !t.text.is_empty() {
        output.push(json!({
            "id": gen_id("msg_"), "type": "message", "role": "assistant", "status": "completed",
            "content": [{ "type": "output_text", "text": t.text }],
        }));
    }
    for tc in &t.tool_calls {
        let (id, name, args) = tool_parts(tc, "call_");
        output.push(json!({
            "id": id, "type": "function_call",
            "call_id": tc.get("id").cloned().unwrap_or(Value::Null),
            "name": name, "arguments": args,
        }));
    }
    json!({
        "id": gen_id("resp_"), "object": "response", "created_at": now_secs(),
        "status": "completed", "model": t.model, "output": output,
        "usage": json!({ "input_tokens": 0, "output_tokens": 0, "total_tokens": 0 }),
    })
}

fn emit_anthropic(t: &Turn) -> Value {
    let mut content: Vec<Value> = Vec::new();
    if !t.text.is_empty() {
        content.push(json!({ "type": "text", "text": t.text }));
    }
    for tc in &t.tool_calls {
        let (id, name, args) = tool_parts(tc, "toolu_");
        content.push(json!({
            "type": "tool_use", "id": id, "name": name,
            "input": parse_args(Some(&args)),
        }));
    }
    let stop = match t.finish_reason.as_str() {
        "length" => "max_tokens",
        "tool_calls" => "tool_use",
        _ => "end_turn",
    };
    json!({
        "id": gen_id("msg_"), "type": "message", "role": "assistant", "model": t.model,
        "content": content, "stop_reason": stop, "usage": { "input_tokens": 0, "output_tokens": 0 },
    })
}

fn emit_gemini(t: &Turn) -> Value {
    let mut parts: Vec<Value> = Vec::new();
    if !t.text.is_empty() {
        parts.push(json!({ "text": t.text }));
    }
    for tc in &t.tool_calls {
        let (_, name, args) = tool_parts(tc, "call_");
        parts.push(json!({ "functionCall": { "name": name, "args": parse_args(Some(&args)) }}));
    }
    json!({
        "candidates": [{ "content": { "parts": parts, "role": "model" }, "finishReason": "STOP", "index": 0 }],
        "modelVersion": t.model,
    })
}

// ── serve engine ────────────────────────────────────────────────────

/// Provider wire protocol a route speaks, so a turn can be emitted as either a
/// JSON body or an SSE stream in that protocol.
#[derive(Clone, Copy)]
enum Wire {
    Chat,
    Responses,
    Anthropic,
    Gemini,
}

/// Advance the FIFO and return the turn for this call (owned, so the empty-fixture
/// sentinel is handled uniformly). Past the last recorded turn, repeat the final
/// one rather than erroring: a *real* gateway in front of replay (full-stack mode)
/// can issue more upstream calls than the fixture has turns — retries, boot-time
/// probes, an agent that re-asks — and the recorded run stays faithful only if
/// those extra calls keep seeing the recorded final answer. An empty fixture has
/// nothing to repeat, so fall back to a benign sentinel.
fn pick_turn(turns: &[Turn]) -> Turn {
    let idx = CALL_INDEX.fetch_add(1, Ordering::SeqCst);
    match select_turn(turns, idx) {
        Some(t) => {
            let repeat = if idx >= turns.len() {
                " (repeat last)"
            } else {
                ""
            };
            eprintln!(
                "[replay] {}/{}{repeat}: {} chars, {} tool_calls",
                idx + 1,
                turns.len(),
                t.text.len(),
                t.tool_calls.len()
            );
            t.clone()
        }
        None => {
            eprintln!("[replay] empty fixture, call {idx}: serving sentinel");
            Turn {
                text: "REPLAY_EXHAUSTED".into(),
                tool_calls: Vec::new(),
                finish_reason: "stop".into(),
                model: "replay".into(),
            }
        }
    }
}

/// Pick the turn for call `idx`: the recorded turn in order, or — once past the
/// last — the final recorded turn (faithful repeat). `None` only for an empty
/// fixture. Split out from `pick_turn` so it's testable without the global index.
fn select_turn(turns: &[Turn], idx: usize) -> Option<&Turn> {
    turns.get(idx).or_else(|| turns.last())
}

fn route(method: &tiny_http::Method, path: &str) -> Option<Wire> {
    if method != &tiny_http::Method::Post {
        return None;
    }
    match path {
        "/v1/chat/completions" | "/openai/v1/chat/completions" => Some(Wire::Chat),
        "/v1/messages" | "/anthropic/v1/messages" => Some(Wire::Anthropic),
        "/v1/responses" | "/openai/v1/responses" => Some(Wire::Responses),
        p if p.starts_with("/v1beta/models/") || p.starts_with("/genai/v1beta/models/") => {
            Some(Wire::Gemini)
        }
        _ => None,
    }
}

fn emit_json(wire: Wire, t: &Turn) -> Value {
    match wire {
        Wire::Chat => emit_chat(t),
        Wire::Responses => emit_responses(t),
        Wire::Anthropic => emit_anthropic(t),
        Wire::Gemini => emit_gemini(t),
    }
}

fn emit_sse(wire: Wire, t: &Turn) -> String {
    match wire {
        Wire::Chat => sse_chat(t),
        Wire::Responses => sse_responses(t),
        Wire::Anthropic => sse_anthropic(t),
        Wire::Gemini => sse_gemini(t),
    }
}

/// Does the caller's request ask for a streamed response? Gemini signals
/// streaming in the path (`:streamGenerateContent`), the others in the body
/// (`stream: true`).
fn wants_stream(body: &str, path: &str) -> bool {
    if path.contains(":streamGenerateContent") {
        return true;
    }
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v.get("stream").and_then(Value::as_bool))
        .unwrap_or(false)
}

/// Emit SSE only when streaming is *both* requested and enabled. SSE is a
/// full-stack need: a real gateway (bifrost) forwards the client's `stream: true`
/// to the upstream and *requires* it to actually stream ("provider returned
/// non-SSE response for streaming request"). The lean path (agent ↔ replay
/// direct) was deliberately built around non-stream replies and is verified that
/// way, so SSE stays off there. The full-stack overlay sets `REPLAY_SSE=1`; lean
/// does not — so a streaming request to a lean replay gets the same JSON it
/// always did.
fn should_stream(sse_enabled: bool, body: &str, path: &str) -> bool {
    sse_enabled && wants_stream(body, path)
}

/// Whether SSE is enabled for this server (`REPLAY_SSE` set non-empty / non-"0").
fn sse_enabled() -> bool {
    std::env::var("REPLAY_SSE")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}

fn sse_event(out: &mut String, event: Option<&str>, data: &Value) {
    if let Some(ev) = event {
        out.push_str("event: ");
        out.push_str(ev);
        out.push('\n');
    }
    out.push_str("data: ");
    out.push_str(&data.to_string());
    out.push_str("\n\n");
}

/// OpenAI Chat Completions SSE: role chunk, one content delta, tool-call deltas,
/// a final finish_reason chunk, then `[DONE]`. Chunk granularity is irrelevant
/// to the client, so the whole recorded text rides one delta.
fn sse_chat(t: &Turn) -> String {
    let id = gen_id("chatcmpl-");
    let created = now_secs();
    let chunk = |delta: Value, finish: Value| {
        json!({
            "id": id, "object": "chat.completion.chunk", "created": created, "model": t.model,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }],
        })
    };
    let mut out = String::new();
    sse_event(
        &mut out,
        None,
        &chunk(json!({ "role": "assistant" }), Value::Null),
    );
    if !t.text.is_empty() {
        sse_event(
            &mut out,
            None,
            &chunk(json!({ "content": t.text }), Value::Null),
        );
    }
    for (i, tc) in t.tool_calls.iter().enumerate() {
        let (id, name, args) = tool_parts(tc, "call_");
        let delta = json!({ "tool_calls": [{
            "index": i, "id": id, "type": "function",
            "function": { "name": name, "arguments": args },
        }] });
        sse_event(&mut out, None, &chunk(delta, Value::Null));
    }
    sse_event(&mut out, None, &chunk(json!({}), json!(t.finish_reason)));
    // Final usage chunk (empty choices) — bifrost maps this to the client's
    // provider usage; without it claude-code reads an undefined `input_tokens`.
    sse_event(
        &mut out,
        None,
        &json!({
            "id": id, "object": "chat.completion.chunk", "created": created, "model": t.model,
            "choices": [], "usage": usage_openai(),
        }),
    );
    out.push_str("data: [DONE]\n\n");
    out
}

/// Anthropic Messages SSE: message_start, a text content block (start/delta/stop),
/// tool_use blocks, then message_delta + message_stop.
fn sse_anthropic(t: &Turn) -> String {
    let id = gen_id("msg_");
    let stop = match t.finish_reason.as_str() {
        "length" => "max_tokens",
        "tool_calls" => "tool_use",
        _ => "end_turn",
    };
    let mut out = String::new();
    sse_event(
        &mut out,
        Some("message_start"),
        &json!({ "type": "message_start", "message": {
            "id": id, "type": "message", "role": "assistant", "model": t.model,
            "content": [], "stop_reason": Value::Null, "stop_sequence": Value::Null,
            "usage": { "input_tokens": 0, "output_tokens": 0 },
        }}),
    );
    let mut block = 0;
    if !t.text.is_empty() {
        sse_event(
            &mut out,
            Some("content_block_start"),
            &json!({ "type": "content_block_start", "index": block, "content_block": { "type": "text", "text": "" }}),
        );
        sse_event(
            &mut out,
            Some("content_block_delta"),
            &json!({ "type": "content_block_delta", "index": block, "delta": { "type": "text_delta", "text": t.text }}),
        );
        sse_event(
            &mut out,
            Some("content_block_stop"),
            &json!({ "type": "content_block_stop", "index": block }),
        );
        block += 1;
    }
    for tc in &t.tool_calls {
        let (id, name, args) = tool_parts(tc, "toolu_");
        let input = parse_args(Some(&args));
        sse_event(
            &mut out,
            Some("content_block_start"),
            &json!({ "type": "content_block_start", "index": block, "content_block": {
                "type": "tool_use", "id": id, "name": name, "input": {},
            }}),
        );
        sse_event(
            &mut out,
            Some("content_block_delta"),
            &json!({ "type": "content_block_delta", "index": block, "delta": { "type": "input_json_delta", "partial_json": input.to_string() }}),
        );
        sse_event(
            &mut out,
            Some("content_block_stop"),
            &json!({ "type": "content_block_stop", "index": block }),
        );
        block += 1;
    }
    sse_event(
        &mut out,
        Some("message_delta"),
        &json!({ "type": "message_delta", "delta": { "stop_reason": stop, "stop_sequence": Value::Null }, "usage": { "output_tokens": 0 }}),
    );
    sse_event(
        &mut out,
        Some("message_stop"),
        &json!({ "type": "message_stop" }),
    );
    out
}

/// OpenAI Responses SSE: created, one output_text delta + done, completed.
fn sse_responses(t: &Turn) -> String {
    let id = gen_id("resp_");
    let full = emit_responses(t);
    let mut out = String::new();
    sse_event(
        &mut out,
        Some("response.created"),
        &json!({ "type": "response.created", "response": { "id": id, "status": "in_progress" }}),
    );
    if !t.text.is_empty() {
        sse_event(
            &mut out,
            Some("response.output_text.delta"),
            &json!({ "type": "response.output_text.delta", "delta": t.text }),
        );
    }
    sse_event(
        &mut out,
        Some("response.completed"),
        &json!({ "type": "response.completed", "response": full }),
    );
    out
}

/// Gemini streamGenerateContent SSE: the same candidate payload as a single chunk.
fn sse_gemini(t: &Turn) -> String {
    let mut out = String::new();
    sse_event(&mut out, None, &emit_gemini(t));
    out
}

fn serve(port: u16, turns: Vec<Turn>) -> ! {
    let addr = format!("0.0.0.0:{port}");
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("[replay] bind {addr} failed: {e}");
        std::process::exit(1);
    });
    let sse = sse_enabled();
    eprintln!("[replay] serving on {addr} (sse={sse})");
    // Single-threaded on purpose: requests are served strictly in arrival order
    // so the recorded turns replay deterministically (FIFO).
    for mut req in server.incoming_requests() {
        let path = req.url().split('?').next().unwrap_or("").to_string();
        let resp = match route(req.method(), &path) {
            Some(wire) => {
                let mut body = String::new();
                let _ = req.as_reader().read_to_string(&mut body);
                let t = pick_turn(&turns);
                let (ctype, data) = if should_stream(sse, &body, &path) {
                    ("text/event-stream", emit_sse(wire, &t).into_bytes())
                } else {
                    (
                        "application/json",
                        serde_json::to_vec(&emit_json(wire, &t)).unwrap_or_else(|_| b"{}".to_vec()),
                    )
                };
                let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes())
                    .expect("static header");
                tiny_http::Response::from_data(data).with_header(header)
            }
            // /health, /, and any unmatched request: a plain-text "ok" (text/plain
            // via from_string, NOT application/json — a JSON client must not parse it).
            None => tiny_http::Response::from_string("ok"),
        };
        let _ = req.respond(resp);
    }
    std::process::exit(0);
}

/// `replay-server health` — the scratch image has no shell/curl, so the binary
/// probes its own /health endpoint for the Docker HEALTHCHECK.
fn health_probe() -> i32 {
    let port = env_port();
    let Ok(mut s) = TcpStream::connect((Ipv4Addr::LOCALHOST, port)) else {
        return 1;
    };
    let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
    if s.write_all(b"GET /health HTTP/1.0\r\nHost: localhost\r\n\r\n")
        .is_err()
    {
        return 1;
    }
    let mut buf = String::new();
    let _ = s.read_to_string(&mut buf);
    i32::from(!buf.starts_with("HTTP/1.0 200") && !buf.starts_with("HTTP/1.1 200"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_line(span_attrs: Value, status_code: i64, start: &str) -> String {
        json!({"resourceSpans":[{"scopeSpans":[{"spans":[
            {"startTimeUnixNano": start, "status": {"code": status_code}, "attributes": span_attrs}
        ]}]}]})
        .to_string()
    }

    fn attr(key: &str, s: &str) -> Value {
        json!({"key": key, "value": {"stringValue": s}})
    }

    #[test]
    fn litellm_shape_one_turn() {
        // litellm OTEL-1.38: parts + tool_calls, one span with a response id.
        let out = json!([{"role":"assistant","parts":[{"type":"text","content":"HELLO_LITELLM"}],"finish_reason":"stop"}]).to_string();
        let line = one_line(
            json!([
                attr("gen_ai.output.messages", &out),
                attr("gen_ai.response.id", "chatcmpl-A"),
                attr("gen_ai.response.model", "gpt-5.4"),
            ]),
            1,
            "100",
        );
        let turns = parse_turns(&line);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].text, "HELLO_LITELLM");
        assert_eq!(turns[0].finish_reason, "stop");
        assert_eq!(turns[0].model, "gpt-5.4");
    }

    #[test]
    fn bifrost_shape_dedups_two_spans_to_one() {
        // bifrost: a summary HTTP span (raw text, no id) + an llm.call span
        // (plain {role,content} array + id). Must yield exactly ONE turn.
        let http = one_line(
            json!([
            attr("gen_ai.output.messages", "HELLO_BIFROST"), // raw text -> dropped
        ]),
            1,
            "100",
        );
        let llmcall = one_line(
            json!([
                attr(
                    "gen_ai.output.messages",
                    &json!([{"role":"assistant","content":"HELLO_BIFROST"}]).to_string()
                ),
                attr("gen_ai.response.id", "chatcmpl-B"),
                attr("gen_ai.response.finish_reason", "stop"),
                attr("gen_ai.response.model", "gpt-5.4"),
            ]),
            1,
            "200",
        );
        let turns = parse_turns(&format!("{http}\n{llmcall}"));
        assert_eq!(turns.len(), 1, "two bifrost spans must dedup to one turn");
        assert_eq!(turns[0].text, "HELLO_BIFROST");
        assert_eq!(turns[0].finish_reason, "stop");
    }

    fn turn(text: &str) -> Turn {
        Turn {
            text: text.into(),
            tool_calls: Vec::new(),
            finish_reason: "stop".into(),
            model: "m".into(),
        }
    }

    #[test]
    fn select_turn_repeats_last_past_the_end() {
        let turns = vec![turn("A"), turn("B")];
        // In order...
        assert_eq!(select_turn(&turns, 0).unwrap().text, "A");
        assert_eq!(select_turn(&turns, 1).unwrap().text, "B");
        // ...then the last turn repeats for every extra call (a real gateway in
        // front can over-call the fixture; the agent must keep seeing the answer).
        assert_eq!(select_turn(&turns, 2).unwrap().text, "B");
        assert_eq!(select_turn(&turns, 99).unwrap().text, "B");
    }

    #[test]
    fn select_turn_none_only_when_empty() {
        assert!(select_turn(&[], 0).is_none());
    }

    #[test]
    fn should_stream_gated_by_sse_enabled() {
        // A streaming request streams only when SSE is enabled (full-stack);
        // disabled (lean), the same request gets non-stream JSON.
        assert!(should_stream(
            true,
            r#"{"stream":true}"#,
            "/v1/chat/completions"
        ));
        assert!(!should_stream(
            false,
            r#"{"stream":true}"#,
            "/v1/chat/completions"
        ));
        // Non-streaming request never streams, regardless of the gate.
        assert!(!should_stream(
            true,
            r#"{"stream":false}"#,
            "/v1/chat/completions"
        ));
    }

    #[test]
    fn wants_stream_from_body_or_gemini_path() {
        assert!(wants_stream(r#"{"stream":true}"#, "/v1/chat/completions"));
        assert!(!wants_stream(r#"{"stream":false}"#, "/v1/chat/completions"));
        assert!(!wants_stream(r#"{"messages":[]}"#, "/v1/chat/completions"));
        assert!(!wants_stream("not json", "/v1/chat/completions"));
        // Gemini signals streaming in the path, not the body.
        assert!(wants_stream("{}", "/v1beta/models/x:streamGenerateContent"));
        assert!(!wants_stream("{}", "/v1beta/models/x:generateContent"));
    }

    #[test]
    fn sse_chat_frames_text_and_terminates() {
        let sse = sse_chat(&turn("HELLO"));
        // SSE framing: every event is `data: ...\n\n`, terminated by [DONE].
        assert!(sse.contains("\"delta\":{\"role\":\"assistant\"}"));
        assert!(sse.contains("\"content\":\"HELLO\""));
        assert!(sse.contains("\"finish_reason\":\"stop\""));
        assert!(sse.trim_end().ends_with("data: [DONE]"));
        for ev in sse.split("\n\n").filter(|e| !e.is_empty()) {
            assert!(ev.starts_with("data: "), "chat SSE event not framed: {ev}");
        }
    }

    #[test]
    fn openai_emitters_carry_usage() {
        // A real gateway maps this onto the client's usage; claude-code reads
        // `usage.input_tokens` and crashes if usage is absent.
        assert!(emit_chat(&turn("x"))["usage"].is_object());
        assert!(emit_responses(&turn("x"))["usage"].is_object());
        assert!(sse_chat(&turn("x")).contains("\"usage\""));
    }

    #[test]
    fn sse_responses_and_gemini_carry_text() {
        assert!(sse_responses(&turn("ANS")).contains("ANS"));
        assert!(sse_responses(&turn("ANS")).contains("response.completed"));
        assert!(sse_gemini(&turn("ANS")).contains("ANS"));
    }

    #[test]
    fn sse_chat_streams_tool_calls() {
        let t = Turn {
            text: String::new(),
            tool_calls: vec![json!({"id":"c1","function":{"name":"read","arguments":"{}"}})],
            finish_reason: "tool_calls".into(),
            model: "m".into(),
        };
        let sse = sse_chat(&t);
        assert!(sse.contains("\"tool_calls\""));
        assert!(sse.contains("\"name\":\"read\""));
        assert!(sse.contains("\"finish_reason\":\"tool_calls\""));
        assert!(sse.trim_end().ends_with("data: [DONE]"));
    }

    #[test]
    fn tool_parts_fills_defaults_and_passes_through() {
        let (id, name, args) = tool_parts(&json!({}), "call_");
        assert!(id.as_str().unwrap().starts_with("call_"));
        assert_eq!(name, json!(""));
        assert_eq!(args, json!("{}"));
        let (id2, name2, args2) = tool_parts(
            &json!({"id":"x","function":{"name":"n","arguments":"{\"a\":1}"}}),
            "call_",
        );
        assert_eq!(id2, json!("x"));
        assert_eq!(name2, json!("n"));
        assert_eq!(args2, json!("{\"a\":1}"));
    }

    #[test]
    fn sse_anthropic_has_message_lifecycle() {
        let sse = sse_anthropic(&turn("HI"));
        assert!(sse.contains("event: message_start"));
        assert!(sse.contains("text_delta") && sse.contains("\"text\":\"HI\""));
        assert!(sse.contains("event: message_stop"));
    }

    #[test]
    fn health_detected_by_arg_or_argv0() {
        // explicit `server health`
        assert!(is_health_args("/opt/gateway/server", Some("health")));
        // invoked AS /opt/gateway/health (gateway-contract drop-in, no args)
        assert!(is_health_args("/opt/gateway/health", None));
        assert!(is_health_args("health", None));
        // normal serve invocations are not health probes
        assert!(!is_health_args("/opt/gateway/server", None));
        assert!(!is_health_args("/opt/gateway/start", None));
    }

    #[test]
    fn error_span_skipped() {
        let out = json!([{"role":"assistant","content":"oops"}]).to_string();
        let line = one_line(
            json!([
                attr("gen_ai.output.messages", &out),
                attr("gen_ai.response.id", "e1")
            ]),
            2,
            "100",
        );
        assert_eq!(parse_turns(&line).len(), 0);
    }

    #[test]
    fn emitters_carry_text_and_tool_calls() {
        let t = Turn {
            text: "hi".into(),
            tool_calls: vec![
                json!({"id":"call_1","type":"function","function":{"name":"read","arguments":"{\"p\":\"x\"}"}}),
            ],
            finish_reason: "tool_calls".into(),
            model: "m".into(),
        };
        let chat = emit_chat(&t);
        assert_eq!(chat["choices"][0]["message"]["content"], "hi");
        assert_eq!(
            chat["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
            "read"
        );
        assert_eq!(chat["choices"][0]["finish_reason"], "tool_calls");

        let anth = emit_anthropic(&t);
        assert_eq!(anth["stop_reason"], "tool_use");
        let blocks = anth["content"].as_array().unwrap();
        assert!(blocks
            .iter()
            .any(|b| b["type"] == "tool_use" && b["input"]["p"] == "x"));

        let gem = emit_gemini(&t);
        assert_eq!(
            gem["candidates"][0]["content"]["parts"][1]["functionCall"]["name"],
            "read"
        );
    }
}
