//! Trajectory health inspection: does every benchmark hand the agent
//! a legitimate task, AND does the agent's actual run look sane?
//!
//! See tests/TRAJECTORY.md for the principle. This file is the
//! rule-based phase, with TWO rule catalogs:
//!
//! - `RULES` — **task half**: regex/length checks applied to the
//!   first user message the agent saw. Catches template leaks,
//!   unresolved env vars, fetch failures, missing files, etc.
//! - `RUN_RULES` — **run half**: checks applied to the full
//!   trajectory (every LiteLLM row). Catches API errors, context
//!   overflows, cost/token runaways, retry storms, empty responses.
//!
//! Run sources:
//!
//!   - Fixtures at tests/run/replay/fixtures/*.traces.jsonl are native OTLP
//!     traces (OTLP/JSON, one ExportTraceServiceRequest per line — what an
//!     otelcol `file` exporter writes). Each gen_ai span is one LLM call;
//!     the first user turn (gen_ai.input.messages) is the task as the agent
//!     saw it. Running the rules against these fixtures validates the rule
//!     catalog itself and spot-checks benchmarks without standing up
//!     containers.
//!
//!   - Any OTel-collected trace that captured gen_ai content can be inspected
//!     by the same code — the rules read OTel semconv, not a bespoke format.
//!
//! Run: cargo test --test task_inspection -- --ignored

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Rule catalog (data, not code) ─────────────────────────────────
//
// Each rule is one row. Severity red = fails the test. Severity yellow
// = warning only. The `id` MUST match an entry in tests/TRAJECTORY.md
// so the doc and the code can't drift.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Severity {
    Red,
    Yellow,
}

struct Rule {
    id: &'static str,
    severity: Severity,
    why: &'static str,
    test: fn(&str) -> bool,
}

impl Rule {
    const fn red(id: &'static str, why: &'static str, test: fn(&str) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Red,
            why,
            test,
        }
    }
    const fn yellow(id: &'static str, why: &'static str, test: fn(&str) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Yellow,
            why,
            test,
        }
    }
}

const RULES: &[Rule] = &[
    // ── Red ─────────────────────────────────────────────────────────
    Rule::red("empty", "user message is empty or whitespace", |t| {
        t.trim().is_empty()
    }),
    Rule::red(
        "env_leaked",
        "unresolved EVAL_* env var in task (substitution failed)",
        |t| {
            t.contains("$EVAL_BENCHMARK")
                || t.contains("${EVAL_BENCHMARK}")
                || t.contains("$EVAL_TASK_ID")
                || t.contains("${EVAL_TASK_ID}")
                || t.contains("${TASK}")
        },
    ),
    Rule::red(
        "template_leak",
        "TEMPLATE.md placeholder leaked into task (author forgot to fill in)",
        |t| {
            [
                "{NAME}",
                "{TASK_PROMPT}",
                "{DATASET}",
                "{SPLIT}",
                "{QUESTION_FIELD}",
                "{ANSWER_FIELD}",
                "{ID_FIELD}",
            ]
            .iter()
            .any(|p| t.contains(p))
        },
    ),
    Rule::red(
        "fetch_failed",
        "task contains evidence of a failed dataset download",
        |t| {
            [
                "404 Not Found",
                "403 Forbidden",
                "HF_TOKEN required",
                "access denied",
                "401 Unauthorized",
            ]
            .iter()
            .any(|s| t.contains(s))
        },
    ),
    Rule::red("file_missing", "task contains filesystem errors", |t| {
        let lc = t.to_lowercase();
        [
            "no such file or directory",
            "permission denied",
            "cannot open",
            "not a directory",
        ]
        .iter()
        .any(|s| lc.contains(s))
    }),
    Rule::red(
        "unresolved_url_var",
        "task contains a URL with an unsubstituted shell var — fetch returned literal",
        |t| t.contains("${") && (t.contains("http://") || t.contains("https://")),
    ),
    Rule::red(
        "todo_or_fixme",
        "task definition contains TODO/FIXME/XXX — unfinished",
        |t| {
            // Match as standalone tokens so we don't flag the word "todo" inside prose
            for tok in ["TODO", "FIXME", "XXX"] {
                for w in t.split(|c: char| !c.is_alphanumeric()) {
                    if w == tok {
                        return true;
                    }
                }
            }
            false
        },
    ),
    Rule::red(
        "control_garbage",
        "task contains non-printable control chars (encoding corruption)",
        |t| {
            t.chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
        },
    ),
    // ── Yellow ──────────────────────────────────────────────────────
    Rule::yellow(
        "too_short",
        "task text < 20 chars — almost certainly a template miss",
        |t| !t.trim().is_empty() && t.trim().len() < 20,
    ),
    Rule::yellow(
        "borderline_short",
        "task is 20-50 chars — suspicious, worth a human glance",
        |t| {
            let len = t.trim().len();
            (20..50).contains(&len)
        },
    ),
    Rule::yellow(
        "runaway_long",
        "> 50k chars — possible template concat runaway",
        |t| t.len() > 50_000,
    ),
    Rule::yellow(
        "repeated_block",
        "same 200-char block repeats 10+ times — possible concat runaway",
        |t| {
            // Cheap heuristic: any 200-char window that appears 10+ times.
            if t.len() < 2_000 {
                return false;
            }
            // Check just the first ~5 positions as probes — not exhaustive,
            // but catches real runaway concatenation without full scan.
            // Index by chars, not bytes: fixtures contain multibyte UTF-8
            // (e.g. CJK) and a byte slice can land mid-codepoint and panic.
            let chars: Vec<char> = t.chars().collect();
            for start in [0, 200, 400, 600, 800] {
                if start + 200 > chars.len() {
                    break;
                }
                let probe: String = chars[start..start + 200].iter().collect();
                if t.matches(&probe).count() >= 10 {
                    return true;
                }
            }
            false
        },
    ),
    Rule::yellow(
        "no_instruction_verb",
        "no instruction verb (solve/write/compute/answer/translate/find/explain/return/print/select)",
        |t| {
            let lc = t.to_lowercase();
            ![
                "solve",
                "write",
                "compute",
                "answer",
                "translate",
                "find",
                "explain",
                "return",
                "print",
                "select",
                "complete",
                "analyze",
                "identify",
                "classify",
                "generate",
                "implement",
                "describe",
                "summarize",
            ]
            .iter()
            .any(|v| lc.contains(v))
        },
    ),
];

// ─── Run-level rule catalog (data, not code) ───────────────────────
//
// Run rules look at the whole trajectory (every row). A row is one gen_ai
// span from the OTLP trace. The catalog IDs MUST match the entries in
// tests/TRAJECTORY.md "Run-half signal catalog".

/// Pre-computed summary of the run — cheaper than re-walking rows in
/// every rule. Computed once per fixture.
///
/// Probe-aware: agents like claude-code and codex bracket their real
/// LLM calls with schema-probe pings (empty messages, no content,
/// sometimes `status=failure` by design). The summary tracks BOTH the
/// raw trajectory and the "substantive" subset — rows that actually
/// produced assistant content. Run rules reason about substantive
/// rows, not the literal last row.
struct RunSummary {
    n_rows: usize,
    n_substantive_rows: usize,
    n_failure_rows: usize,
    /// Status of the last SUBSTANTIVE row (not the literal last row —
    /// the final probe may be a ping). Empty if no substantive rows.
    last_substantive_status: String,
    any_assistant_content_nonempty: bool,
    total_tokens: u64,
    total_cost: f64,
    max_consecutive_identical_prompts: usize,
    /// Any row's error message (status, error_str, error_information).
    any_error_message: String,
    /// Number of rows whose assistant content contains a refusal
    /// phrase (Azure content_filter, safety refusal, "I'm sorry but I
    /// cannot...").
    n_refusal_rows: usize,
    /// Number of rows that hit max_tokens / length truncation
    /// (finish_reason=length or stop_reason=max_tokens).
    n_max_tokens_rows: usize,
    /// The LAST substantive row's assistant content looks like a refusal.
    final_response_is_refusal: bool,
    /// Concatenated first-row user text contains a reference to an
    /// external file the agent is told to read ("see /app/task.txt",
    /// "read the file at ...") — the real task is delegated and the
    /// first-row rule catalog misses it.
    task_delegates_to_file: bool,
    /// Task mentions "attached", "uploaded", "see the spreadsheet",
    /// "image", "document" but no file path exists in /tasks/ or
    /// referenced in the prompt.
    task_references_attachment: bool,
    /// The task appears to require fetching/browsing (contains "search",
    /// "look up", "visit", URL) but there are no tool_calls in any row.
    fetch_required_but_no_tool_calls: bool,
}

struct RunRule {
    id: &'static str,
    severity: Severity,
    why: &'static str,
    test: fn(&RunSummary) -> bool,
}

impl RunRule {
    const fn red(id: &'static str, why: &'static str, test: fn(&RunSummary) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Red,
            why,
            test,
        }
    }
    const fn yellow(id: &'static str, why: &'static str, test: fn(&RunSummary) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Yellow,
            why,
            test,
        }
    }
}

const COST_CAP_USD: f64 = 5.0;
const TOKEN_CAP: u64 = 200_000;
const TURN_CAP: usize = 100;

const RUN_RULES: &[RunRule] = &[
    // ── Red ─────────────────────────────────────────────────────────
    RunRule::red(
        "no_substantive_output",
        "every LLM call produced zero content and zero tool calls — the run said nothing",
        |s| s.n_rows > 0 && !s.any_assistant_content_nonempty,
    ),
    RunRule::red(
        "last_substantive_row_failed",
        "the final LLM call that produced real output ended in status != success",
        |s| s.n_substantive_rows > 0 && s.last_substantive_status != "success",
    ),
    RunRule::red("context_overflow", "context window was exceeded", |s| {
        let lc = s.any_error_message.to_lowercase();
        lc.contains("context_length_exceeded")
            || lc.contains("context window")
            || lc.contains("maximum context length")
    }),
    RunRule::red(
        "auth_failure",
        "an LLM call hit an auth/permission error (401/403/invalid key)",
        |s| {
            let lc = s.any_error_message.to_lowercase();
            lc.contains("401 unauthorized")
                || lc.contains("403 forbidden")
                || lc.contains("invalid api key")
                || lc.contains("authenticationerror")
                || lc.contains("permission denied")
        },
    ),
    // ── Yellow ──────────────────────────────────────────────────────
    RunRule::yellow(
        "cost_runaway",
        "total response_cost exceeds the per-task cap ($5)",
        |s| s.total_cost > COST_CAP_USD,
    ),
    RunRule::yellow(
        "token_runaway",
        "total_tokens exceed the per-task cap (200k)",
        |s| s.total_tokens > TOKEN_CAP,
    ),
    RunRule::yellow(
        "high_turn_count",
        "more than 100 LLM calls for a single task",
        |s| s.n_rows > TURN_CAP,
    ),
    RunRule::yellow(
        "retry_storm",
        "same prompt repeated 5+ times in a row with no material change",
        |s| s.max_consecutive_identical_prompts >= 5,
    ),
    RunRule::yellow(
        "high_substantive_failure_rate",
        "more than half of the substantive LLM calls ended in failure",
        |s| s.n_substantive_rows > 1 && s.n_failure_rows * 2 > s.n_substantive_rows,
    ),
    // ── New rules from the 2026-04-15 trajectory audit walk ────────
    RunRule::red(
        "refusal_final_response",
        "the final substantive assistant turn is a safety refusal — the run never answered the task",
        |s| s.final_response_is_refusal,
    ),
    RunRule::yellow(
        "content_filter_refusal",
        "one or more assistant turns contain a content_filter refusal (rides a valid response body)",
        |s| s.n_refusal_rows > 0,
    ),
    RunRule::yellow(
        "max_tokens_truncation",
        "one or more assistant turns were truncated at max_tokens mid-answer",
        |s| s.n_max_tokens_rows > 0,
    ),
    RunRule::red(
        "task_delegates_to_external_file",
        "first user message is a short pointer to a file (e.g. /app/task.txt) — the task-half rule catalog never saw the real instruction",
        |s| s.task_delegates_to_file,
    ),
    RunRule::yellow(
        "attachment_referenced_but_not_provided",
        "task mentions an attached file / spreadsheet / image / document but no file path is provided",
        |s| s.task_references_attachment,
    ),
    RunRule::yellow(
        "fetch_required_but_no_tool_calls",
        "task requires browsing / searching / fetching a URL but the trace has zero tool_calls",
        |s| s.fetch_required_but_no_tool_calls,
    ),
];

// ─── Engine ────────────────────────────────────────────────────────

#[derive(Debug)]
struct Finding {
    source: String,
    rule: &'static str,
    severity: Severity,
    why: &'static str,
}

fn inspect(source: &str, task: &str) -> Vec<Finding> {
    RULES
        .iter()
        .filter(|r| (r.test)(task))
        .map(|r| Finding {
            source: source.to_string(),
            rule: r.id,
            severity: r.severity,
            why: r.why,
        })
        .collect()
}

fn inspect_run(source: &str, summary: &RunSummary) -> Vec<Finding> {
    RUN_RULES
        .iter()
        .filter(|r| (r.test)(summary))
        .map(|r| Finding {
            source: source.to_string(),
            rule: r.id,
            severity: r.severity,
            why: r.why,
        })
        .collect()
}

// ─── Trajectory parsing from native OTLP traces ────────────────────
//
// A fixture is OTLP/JSON: one ExportTraceServiceRequest per line (what an
// otelcol `file` exporter writes). Each LLM call is a span carrying OTel
// gen_ai semconv attributes. We flatten every span across
// resourceSpans/scopeSpans, order by startTimeUnixNano (the order the agent
// made the calls), and treat each span as one "row".
//
// Content lives in two JSON-string attributes (OTEL GenAI 1.38 shape):
//   - gen_ai.input.messages  : [{role, parts:[{type,content}], tool_calls?}]
//   - gen_ai.output.messages : [{role, parts:[{type,content}], tool_calls?, finish_reason}]
// Failures are span status ERROR (code 2) + error.* attrs + an `exception`
// event. Tokens/cost are gen_ai.usage.* / gen_ai.cost.*.

/// One LLM call, parsed from an OTLP span.
struct SpanCall {
    start_ns: u128,
    status_code: i64, // 0 unset, 1 ok, 2 error (opentelemetry StatusCode)
    status_message: String,
    attrs: HashMap<String, Value>,
    events: Vec<Value>,
}

/// Flatten a span's `attributes: [{key, value:{<type>Value: ...}}]` list into
/// a key->Value map. intValue arrives as a string-encoded int64.
fn flatten_attrs(span: &Value) -> HashMap<String, Value> {
    let mut m = HashMap::new();
    let Some(arr) = span.get("attributes").and_then(Value::as_array) else {
        return m;
    };
    for a in arr {
        let Some(k) = a.get("key").and_then(Value::as_str) else {
            continue;
        };
        let Some(v) = a.get("value") else { continue };
        if let Some(s) = v.get("stringValue").and_then(Value::as_str) {
            m.insert(k.to_string(), Value::String(s.to_string()));
        } else if let Some(iv) = v.get("intValue") {
            if let Some(n) = iv
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| iv.as_i64())
            {
                m.insert(k.to_string(), Value::from(n));
            }
        } else if let Some(d) = v.get("doubleValue").and_then(Value::as_f64) {
            m.insert(k.to_string(), Value::from(d));
        } else if let Some(b) = v.get("boolValue").and_then(Value::as_bool) {
            m.insert(k.to_string(), Value::Bool(b));
        }
    }
    m
}

/// status.code may be numeric (1/2) or the proto-JSON string form.
fn status_code_of(status: Option<&Value>) -> i64 {
    let Some(code) = status.and_then(|s| s.get("code")) else {
        return 0;
    };
    if let Some(n) = code.as_i64() {
        return n;
    }
    match code.as_str() {
        Some("STATUS_CODE_ERROR") => 2,
        Some("STATUS_CODE_OK") => 1,
        _ => 0,
    }
}

/// Load every span from an OTLP/JSON fixture, ordered by start time.
fn load_spans(path: &Path) -> Result<Vec<SpanCall>, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut calls: Vec<SpanCall> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
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
                    let start_ns = span
                        .get("startTimeUnixNano")
                        .and_then(|v| {
                            v.as_str()
                                .and_then(|s| s.parse::<u128>().ok())
                                .or_else(|| v.as_u64().map(u128::from))
                        })
                        .unwrap_or(0);
                    calls.push(SpanCall {
                        start_ns,
                        status_code: status_code_of(span.get("status")),
                        status_message: span
                            .get("status")
                            .and_then(|s| s.get("message"))
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        attrs: flatten_attrs(span),
                        events: span
                            .get("events")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
            }
        }
    }
    calls.sort_by_key(|c| c.start_ns);
    Ok(calls)
}

fn attr_str<'a>(call: &'a SpanCall, key: &str) -> Option<&'a str> {
    call.attrs.get(key).and_then(Value::as_str)
}

/// Parse a gen_ai.*.messages JSON-string attribute into its message array.
fn parse_messages(call: &SpanCall, key: &str) -> Vec<Value> {
    attr_str(call, key)
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

/// Text content of a message's `parts` array. List-content messages pass
/// their original parts through verbatim, so a text part carries `text`
/// (OpenAI chat) or `input_text` (Responses input); string-content messages
/// use the OTEL 1.38 wrapping `{type:text, content:...}`. We read `text` /
/// `input_text` first and only fall back to `content` when `type == "text"`
/// — so tool_result / image blocks (which carry a `content` field but no
/// text) are skipped, matching the original row extraction and keeping
/// per-call tool output out of the user-prompt text.
fn parts_text(msg: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for p in msg
        .get("parts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let t = p
            .get("text")
            .and_then(Value::as_str)
            .or_else(|| p.get("input_text").and_then(Value::as_str))
            .or_else(|| {
                (p.get("type").and_then(Value::as_str) == Some("text"))
                    .then(|| p.get("content").and_then(Value::as_str))
                    .flatten()
            });
        if let Some(t) = t {
            if !t.is_empty() {
                out.push(t.to_string());
            }
        }
    }
    out
}

fn msg_has_tool_calls(msg: &Value) -> bool {
    msg.get("tool_calls")
        .and_then(Value::as_array)
        .map(|tc| !tc.is_empty())
        .unwrap_or(false)
}

// The first non-empty user turn the agent saw is the task. Some agents send a
// schema-probe call first, so we walk spans in order and take the first span
// whose gen_ai.input.messages has non-empty user content.

fn span_user_text(call: &SpanCall) -> String {
    let mut parts: Vec<String> = Vec::new();
    for msg in parse_messages(call, "gen_ai.input.messages") {
        if msg.get("role").and_then(Value::as_str) != Some("user") {
            continue;
        }
        parts.extend(parts_text(&msg));
    }
    parts.join("\n\n")
}

fn extract_user_text_from_fixture(path: &Path) -> Result<String, String> {
    for call in &load_spans(path)? {
        let text = span_user_text(call);
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }
    Err(format!(
        "{}: no span had a non-empty user message",
        path.display()
    ))
}

// ─── Run summary builder ──────────────────────────────────────────
//
// Walks every row in the fixture and pre-computes the fields the run
// rules need. A run rule is just a closure over RunSummary, so the
// expensive work happens once per fixture instead of once per rule.

fn span_assistant_content(call: &SpanCall) -> String {
    // "Did the assistant say anything substantive?" — text from the
    // output.messages parts plus a marker for any tool call. Non-empty after
    // trim => substantive.
    let mut parts: Vec<String> = Vec::new();
    for msg in parse_messages(call, "gen_ai.output.messages") {
        parts.extend(parts_text(&msg));
        if msg_has_tool_calls(&msg) {
            parts.push("<tool_calls>".into());
        }
    }
    parts.join("\n")
}

fn span_error_message(call: &SpanCall) -> String {
    // Every error signal on the span: status message, error.* attributes, and
    // the type/message of any `exception` event.
    let mut parts: Vec<String> = Vec::new();
    if !call.status_message.is_empty() {
        parts.push(call.status_message.clone());
    }
    for key in ["error.type", "error.message"] {
        if let Some(s) = attr_str(call, key)
            && !s.is_empty()
        {
            parts.push(s.to_string());
        }
    }
    for ev in &call.events {
        if ev.get("name").and_then(Value::as_str) != Some("exception") {
            continue;
        }
        for a in ev
            .get("attributes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let k = a.get("key").and_then(Value::as_str).unwrap_or("");
            if (k == "exception.message" || k == "exception.type")
                && let Some(m) = a
                    .get("value")
                    .and_then(|v| v.get("stringValue"))
                    .and_then(Value::as_str)
                && !m.is_empty()
            {
                parts.push(m.to_string());
            }
        }
    }
    parts.join(" | ")
}

// ─── Row-level detectors for the run-half rules ────────────────────
//
// Each of these reads one row (or the assistant content we already
// extracted) and returns a bool. Keeping them as tiny free functions
// so summarize_run stays flat and rule additions are one-line edits.

const REFUSAL_PHRASES: &[&str] = &[
    "i'm sorry, but i cannot assist",
    "i'm sorry, but i can't assist",
    "i cannot help with that request",
    "i cannot provide assistance",
    "i cannot comply",
    "unable to assist with that request",
    "against my guidelines",
    "i must refuse",
];

fn content_is_refusal(content: &str) -> bool {
    let lc = content.to_lowercase();
    REFUSAL_PHRASES.iter().any(|p| lc.contains(p))
}

fn span_hit_max_tokens(call: &SpanCall) -> bool {
    // Truncation, from gen_ai.response.finish_reasons or the per-message
    // finish_reason inside output.messages.
    let is_trunc = |s: &str| s == "length" || s == "max_tokens" || s == "max_output_tokens";
    if let Some(raw) = attr_str(call, "gen_ai.response.finish_reasons")
        && let Ok(arr) = serde_json::from_str::<Value>(raw)
    {
        for fr in arr.as_array().into_iter().flatten() {
            if fr.as_str().map(is_trunc).unwrap_or(false) {
                return true;
            }
        }
    }
    parse_messages(call, "gen_ai.output.messages")
        .iter()
        .any(|msg| {
            msg.get("finish_reason")
                .and_then(Value::as_str)
                .map(is_trunc)
                .unwrap_or(false)
        })
}

fn span_has_tool_calls(call: &SpanCall) -> bool {
    parse_messages(call, "gen_ai.output.messages")
        .iter()
        .any(msg_has_tool_calls)
}

fn task_delegates_to_file_heuristic(task: &str) -> bool {
    // A short message that points at an in-container path. Task-half
    // rule catalog walks the first user message — if that message is
    // just "see /app/task.txt" the real task is hidden from us.
    let trimmed = task.trim();
    if trimmed.len() > 400 {
        return false;
    }
    let lc = trimmed.to_lowercase();
    let points_at_path = ["/app/", "/tasks/", "/workspace/", "/data/"]
        .iter()
        .any(|p| lc.contains(p));
    let mentions_file_verb = ["read", "see", "open", "load"]
        .iter()
        .any(|v| lc.contains(v));
    points_at_path && mentions_file_verb
}

fn task_references_attachment_heuristic(task: &str) -> bool {
    let lc = task.to_lowercase();
    let phrases = [
        "attached spreadsheet",
        "attached document",
        "attached image",
        "attached file",
        "the attached",
        "the uploaded",
        "uploaded file",
        "see the image",
        "see the spreadsheet",
        "refer to the attached",
    ];
    phrases.iter().any(|p| lc.contains(p))
}

fn task_requires_fetching_heuristic(task: &str) -> bool {
    let lc = task.to_lowercase();
    let action_verbs = [
        "search the web",
        "look up",
        "browse to",
        "visit the",
        "open the following url",
        "fetch the page",
        "scrape",
    ];
    if action_verbs.iter().any(|p| lc.contains(p)) {
        return true;
    }
    // Any http(s) URL in the prompt
    lc.contains("http://") || lc.contains("https://")
}

fn summarize_run(path: &Path) -> Result<RunSummary, String> {
    let calls = load_spans(path)?;

    let mut n_substantive_rows = 0usize;
    let mut n_failure_rows = 0usize;
    let mut n_refusal_rows = 0usize;
    let mut n_max_tokens_rows = 0usize;
    let mut last_substantive_status = String::new();
    let mut last_substantive_content = String::new();
    let mut any_assistant_nonempty = false;
    let mut any_tool_calls = false;
    let mut total_tokens: u64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut any_error_message = String::new();

    // Retry-storm detection: walk adjacent spans' user prompts and
    // track the longest run of identical ones.
    let mut last_prompt = String::new();
    let mut current_streak = 1usize;
    let mut max_streak = 1usize;

    // First user prompt for task-level delegation / attachment checks.
    let mut first_user_text = String::new();

    for call in &calls {
        // OTel StatusCode: 2 = ERROR (a failed call); anything else = ok.
        let status = if call.status_code == 2 {
            "failure"
        } else {
            "success"
        };

        if let Some(t) = call
            .attrs
            .get("gen_ai.usage.total_tokens")
            .and_then(Value::as_u64)
        {
            total_tokens += t;
        }
        // Prefer the rolled-up gen_ai.cost.total_cost; otherwise sum the
        // per-component gen_ai.cost.* attributes.
        total_cost += call
            .attrs
            .get("gen_ai.cost.total_cost")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| {
                call.attrs
                    .iter()
                    .filter(|(k, _)| k.starts_with("gen_ai.cost."))
                    .filter_map(|(_, v)| v.as_f64())
                    .sum()
            });

        let err = span_error_message(call);
        if !err.is_empty() && any_error_message.is_empty() {
            any_error_message = err;
        }

        if span_hit_max_tokens(call) {
            n_max_tokens_rows += 1;
        }
        if span_has_tool_calls(call) {
            any_tool_calls = true;
        }

        let assistant = span_assistant_content(call);
        let substantive = !assistant.trim().is_empty();
        if substantive {
            n_substantive_rows += 1;
            any_assistant_nonempty = true;
            last_substantive_status = status.to_string();
            last_substantive_content = assistant.clone();
            if status != "success" {
                n_failure_rows += 1;
            }
            if content_is_refusal(&assistant) {
                n_refusal_rows += 1;
            }
        }

        let prompt = span_user_text(call);
        if first_user_text.is_empty() && !prompt.trim().is_empty() {
            first_user_text = prompt.clone();
        }
        if !prompt.is_empty() {
            if prompt == last_prompt {
                current_streak += 1;
                if current_streak > max_streak {
                    max_streak = current_streak;
                }
            } else {
                current_streak = 1;
                last_prompt = prompt;
            }
        }
    }

    let final_response_is_refusal =
        !last_substantive_content.is_empty() && content_is_refusal(&last_substantive_content);
    let task_delegates_to_file = task_delegates_to_file_heuristic(&first_user_text);
    let task_references_attachment = task_references_attachment_heuristic(&first_user_text);
    let fetch_required_but_no_tool_calls =
        task_requires_fetching_heuristic(&first_user_text) && !any_tool_calls;

    Ok(RunSummary {
        n_rows: calls.len(),
        n_substantive_rows,
        n_failure_rows,
        last_substantive_status,
        any_assistant_content_nonempty: any_assistant_nonempty,
        total_tokens,
        total_cost,
        max_consecutive_identical_prompts: max_streak,
        any_error_message,
        n_refusal_rows,
        n_max_tokens_rows,
        final_response_is_refusal,
        task_delegates_to_file,
        task_references_attachment,
        fetch_required_but_no_tool_calls,
    })
}

fn fixture_paths() -> Vec<PathBuf> {
    let dir = test_support::repo_root().join("tests/run/replay/fixtures");
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl")
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|n| n.ends_with(".traces.jsonl"))
                .unwrap_or(false)
        {
            out.push(path);
        }
    }
    out.sort();
    out
}

// ─── Tests ─────────────────────────────────────────────────────────
//
// Unit tests for the rule engine use synthetic inputs. The fixture
// sweep reads tests/run/replay/fixtures/ — pure file I/O, completes in ~10ms —
// so it always runs on `cargo test` (no --ignored needed).

#[test]
fn rule_empty_fires_on_whitespace() {
    let fs = inspect("t", "   \n\t  ");
    assert!(fs.iter().any(|f| f.rule == "empty"));
}

#[test]
fn rule_env_leaked_fires_on_unresolved_eval_var() {
    let fs = inspect(
        "t",
        "Solve task $EVAL_TASK_ID from benchmark ${EVAL_BENCHMARK}.",
    );
    assert!(fs.iter().any(|f| f.rule == "env_leaked"));
}

#[test]
fn rule_template_leak_fires_on_placeholder() {
    let fs = inspect("t", "Solve this {NAME} problem: {TASK_PROMPT}");
    assert!(fs.iter().any(|f| f.rule == "template_leak"));
}

#[test]
fn rule_fetch_failed_fires_on_404() {
    let fs = inspect("t", "Task not found: 404 Not Found at huggingface.co/...");
    assert!(fs.iter().any(|f| f.rule == "fetch_failed"));
}

fn blank_summary() -> RunSummary {
    RunSummary {
        n_rows: 5,
        n_substantive_rows: 5,
        n_failure_rows: 0,
        last_substantive_status: "success".to_string(),
        any_assistant_content_nonempty: true,
        total_tokens: 1000,
        total_cost: 0.01,
        max_consecutive_identical_prompts: 1,
        any_error_message: String::new(),
        n_refusal_rows: 0,
        n_max_tokens_rows: 0,
        final_response_is_refusal: false,
        task_delegates_to_file: false,
        task_references_attachment: false,
        fetch_required_but_no_tool_calls: false,
    }
}

#[test]
fn run_rule_refusal_final_response_fires() {
    let mut s = blank_summary();
    s.final_response_is_refusal = true;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "refusal_final_response"));
}

#[test]
fn run_rule_content_filter_fires_on_refusal_row() {
    let mut s = blank_summary();
    s.n_refusal_rows = 1;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "content_filter_refusal"));
}

#[test]
fn run_rule_max_tokens_truncation_fires() {
    let mut s = blank_summary();
    s.n_max_tokens_rows = 3;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "max_tokens_truncation"));
}

#[test]
fn run_rule_task_delegates_to_external_file_fires() {
    let mut s = blank_summary();
    s.task_delegates_to_file = true;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "task_delegates_to_external_file"));
}

#[test]
fn heuristic_refusal_detects_azure_phrase() {
    assert!(content_is_refusal(
        "I'm sorry, but I cannot assist with that request."
    ));
    assert!(!content_is_refusal("I am happy to help with the task."));
}

#[test]
fn heuristic_task_delegates_to_file_detects_short_pointer() {
    assert!(task_delegates_to_file_heuristic(
        "Please read the task instructions at /app/task.txt and solve the problem."
    ));
    assert!(!task_delegates_to_file_heuristic(
        "Solve this aime problem: Let P(x) be a polynomial..."
    ));
}

#[test]
fn run_rule_last_substantive_failed_fires() {
    let mut s = blank_summary();
    s.last_substantive_status = "failure".to_string();
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "last_substantive_row_failed"));
}

#[test]
fn run_rule_no_substantive_output_fires_on_empty() {
    let mut s = blank_summary();
    s.any_assistant_content_nonempty = false;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "no_substantive_output"));
}

#[test]
fn run_rule_cost_runaway_fires_above_cap() {
    let mut s = blank_summary();
    s.total_cost = 6.0;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "cost_runaway"));
}

#[test]
fn run_rule_token_runaway_fires_above_cap() {
    let mut s = blank_summary();
    s.total_tokens = 250_000;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "token_runaway"));
}

#[test]
fn run_rule_retry_storm_fires_on_5() {
    let mut s = blank_summary();
    s.max_consecutive_identical_prompts = 5;
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "retry_storm"));
}

#[test]
fn run_rule_context_overflow_fires_on_keyword() {
    let mut s = blank_summary();
    s.any_error_message = "Error: context_length_exceeded (200000)".to_string();
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(fs.iter().any(|r| r.id == "context_overflow"));
}

#[test]
fn run_rule_clean_summary_produces_no_findings() {
    let s = blank_summary();
    let fs: Vec<&RunRule> = RUN_RULES.iter().filter(|r| (r.test)(&s)).collect();
    assert!(
        fs.is_empty(),
        "expected clean run, got: {:?}",
        fs.iter().map(|r| r.id).collect::<Vec<_>>()
    );
}

#[test]
fn clean_task_produces_no_findings() {
    let clean = "Solve the following AIME problem. Print only the answer as a single integer.\n\n\
                 Quadratic polynomials P(x) and Q(x) have leading coefficients 2 and -2...";
    let fs = inspect("t", clean);
    assert!(fs.is_empty(), "expected clean task, got: {fs:?}");
}

/// Fixtures marked broken in tests/run/replay/fixtures/broken.json are
/// known-failing runs (refusals, wrong answers, content filters, etc.)
/// pending re-recording. Their findings are reported as info but do
/// not fail the sweep — the rules still fire on them, so future
/// regressions are visible, but they don't block CI.
fn broken_fixture_set() -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let Ok(raw) =
        fs::read_to_string(test_support::repo_root().join("tests/run/replay/fixtures/broken.json"))
    else {
        return out;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return out;
    };
    if let Some(list) = v.get("broken").and_then(Value::as_array) {
        for item in list {
            if let Some(name) = item.get("fixture").and_then(Value::as_str) {
                out.insert(name.to_string());
            }
        }
    }
    out
}

#[test]
fn inspect_every_existing_fixture() {
    let fixtures = fixture_paths();
    assert!(
        !fixtures.is_empty(),
        "no fixtures found under tests/run/replay/fixtures/"
    );
    let broken = broken_fixture_set();

    let mut all: Vec<Finding> = Vec::new();
    let mut broken_findings: Vec<Finding> = Vec::new();
    let mut extraction_errors: Vec<String> = Vec::new();

    for fixture in &fixtures {
        let source = fixture
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let is_broken = broken.contains(&source);

        let mut per_fixture: Vec<Finding> = Vec::new();

        // Task half
        match extract_user_text_from_fixture(fixture) {
            Ok(task) => per_fixture.extend(inspect(&source, &task)),
            Err(e) => extraction_errors.push(e),
        }

        // Run half
        match summarize_run(fixture) {
            Ok(summary) => per_fixture.extend(inspect_run(&source, &summary)),
            Err(e) => extraction_errors.push(e),
        }

        if is_broken {
            broken_findings.extend(per_fixture);
        } else {
            all.extend(per_fixture);
        }
    }

    let red: Vec<&Finding> = all.iter().filter(|f| f.severity == Severity::Red).collect();
    let yellow: Vec<&Finding> = all
        .iter()
        .filter(|f| f.severity == Severity::Yellow)
        .collect();

    let live_count = fixtures.len() - broken.len();
    eprintln!(
        "\n─── trajectory inspection over {} fixtures ({} live, {} marked broken) ───",
        fixtures.len(),
        live_count,
        broken.len()
    );
    if !broken_findings.is_empty() {
        eprintln!(
            "\n{} findings on known-broken fixtures (informational, not blocking):",
            broken_findings.len()
        );
        for f in &broken_findings {
            eprintln!(
                "  [broken] {} ({:?} {}): {}",
                f.source, f.severity, f.rule, f.why
            );
        }
    }
    if !yellow.is_empty() {
        eprintln!("\n{} yellow findings on live fixtures:", yellow.len());
        for f in &yellow {
            eprintln!("  {} ({}): {}", f.source, f.rule, f.why);
        }
    }
    if !extraction_errors.is_empty() {
        eprintln!("\n{} extraction errors:", extraction_errors.len());
        for e in &extraction_errors {
            eprintln!("  {e}");
        }
    }
    if red.is_empty() && extraction_errors.is_empty() {
        eprintln!(
            "\n✓ all {live_count} live fixtures produced a healthy task ({} yellow warnings)",
            yellow.len()
        );
        return;
    }

    let mut msg = String::new();
    if !red.is_empty() {
        msg.push_str(&format!("\n{} red findings on live fixtures:\n", red.len()));
        for f in &red {
            msg.push_str(&format!("  {} ({}): {}\n", f.source, f.rule, f.why));
        }
    }
    if !extraction_errors.is_empty() {
        msg.push_str(&format!(
            "\n{} extraction errors:\n",
            extraction_errors.len()
        ));
        for e in &extraction_errors {
            msg.push_str(&format!("  {e}\n"));
        }
    }
    panic!("{msg}");
}
