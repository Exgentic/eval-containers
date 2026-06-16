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
struct Turn {
    text: String,
    tool_calls: Vec<Value>,
    finish_reason: String,
    model: String,
}

static CALL_INDEX: AtomicUsize = AtomicUsize::new(0);
static ID_SEQ: AtomicUsize = AtomicUsize::new(0);

fn main() {
    if std::env::args().nth(1).as_deref() == Some("health") {
        std::process::exit(health_probe());
    }
    let port = env_port();
    let path = std::env::var("REPLAY_TRACES").unwrap_or_else(|_| "/data/traces.jsonl".into());
    let turns = load_turns(&path);
    eprintln!("[replay] loaded {} gen_ai turns from {path}", turns.len());
    serve(port, turns);
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
    })
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
        let func = tc.get("function");
        output.push(json!({
            "id": tc.get("id").cloned().unwrap_or_else(|| json!(gen_id("call_"))),
            "type": "function_call",
            "call_id": tc.get("id").cloned().unwrap_or(Value::Null),
            "name": func.and_then(|f| f.get("name")).cloned().unwrap_or_else(|| json!("")),
            "arguments": func.and_then(|f| f.get("arguments")).cloned().unwrap_or_else(|| json!("{}")),
        }));
    }
    json!({
        "id": gen_id("resp_"), "object": "response", "created_at": now_secs(),
        "status": "completed", "model": t.model, "output": output,
    })
}

fn emit_anthropic(t: &Turn) -> Value {
    let mut content: Vec<Value> = Vec::new();
    if !t.text.is_empty() {
        content.push(json!({ "type": "text", "text": t.text }));
    }
    for tc in &t.tool_calls {
        let func = tc.get("function");
        content.push(json!({
            "type": "tool_use",
            "id": tc.get("id").cloned().unwrap_or_else(|| json!(gen_id("toolu_"))),
            "name": func.and_then(|f| f.get("name")).cloned().unwrap_or_else(|| json!("")),
            "input": parse_args(func.and_then(|f| f.get("arguments"))),
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
        let func = tc.get("function");
        parts.push(json!({ "functionCall": {
            "name": func.and_then(|f| f.get("name")).cloned().unwrap_or_else(|| json!("")),
            "args": parse_args(func.and_then(|f| f.get("arguments"))),
        }}));
    }
    json!({
        "candidates": [{ "content": { "parts": parts, "role": "model" }, "finishReason": "STOP", "index": 0 }],
        "modelVersion": t.model,
    })
}

// ── serve engine ────────────────────────────────────────────────────

/// Pull the next recorded turn (FIFO) and emit it; on exhaustion serve a benign
/// sentinel so the agent doesn't crash on a missing reply.
fn next_turn(turns: &[Turn], emit: fn(&Turn) -> Value) -> Value {
    let idx = CALL_INDEX.fetch_add(1, Ordering::SeqCst);
    if let Some(t) = turns.get(idx) {
        eprintln!(
            "[replay] {}/{}: {} chars, {} tool_calls",
            idx + 1,
            turns.len(),
            t.text.len(),
            t.tool_calls.len()
        );
        emit(t)
    } else {
        eprintln!("[replay] EXHAUSTED after {idx}");
        emit(&Turn {
            text: "REPLAY_EXHAUSTED".into(),
            tool_calls: Vec::new(),
            finish_reason: "stop".into(),
            model: "replay".into(),
        })
    }
}

fn route(method: &tiny_http::Method, path: &str) -> Option<fn(&Turn) -> Value> {
    if method != &tiny_http::Method::Post {
        return None;
    }
    match path {
        "/v1/chat/completions" | "/openai/v1/chat/completions" => Some(emit_chat),
        "/v1/messages" | "/anthropic/v1/messages" => Some(emit_anthropic),
        "/v1/responses" | "/openai/v1/responses" => Some(emit_responses),
        p if p.starts_with("/v1beta/models/") || p.starts_with("/genai/v1beta/models/") => {
            Some(emit_gemini)
        }
        _ => None,
    }
}

fn serve(port: u16, turns: Vec<Turn>) -> ! {
    let addr = format!("0.0.0.0:{port}");
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("[replay] bind {addr} failed: {e}");
        std::process::exit(1);
    });
    eprintln!("[replay] serving on {addr}");
    // Single-threaded on purpose: requests are served strictly in arrival order
    // so the recorded turns replay deterministically (FIFO).
    for req in server.incoming_requests() {
        let path = req.url().split('?').next().unwrap_or("").to_string();
        let resp = match route(req.method(), &path) {
            Some(emit) => {
                let body =
                    serde_json::to_vec(&next_turn(&turns, emit)).unwrap_or_else(|_| b"{}".to_vec());
                let ctype =
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .expect("static header");
                tiny_http::Response::from_data(body).with_header(ctype)
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
