"""Replay model: serves recorded LLM responses for deterministic testing.

Reads an OpenTelemetry trace file (OTLP/JSON, one ExportTraceServiceRequest per
line — exactly what an otelcol `file` exporter writes) and serves the recorded
gen_ai responses in order. From the eval container's perspective this is
indistinguishable from a real LiteLLM proxy.

Input contract — native OTel gen_ai semconv (OTEL GenAI 1.38):
  Each LLM call is a span carrying `gen_ai.output.messages` (a JSON *string* of
  `[{role, parts:[{type,content}], tool_calls, finish_reason}]`) plus
  `gen_ai.response.model`. Spans are flattened across resourceSpans/scopeSpans,
  ordered by startTimeUnixNano, and ERROR spans (status.code == 2) are skipped —
  leaving the run's successful turns, in order. Because the input is plain OTLP,
  ANY otel-collected trace that captured gen_ai content can be replayed, not
  just traces produced by this repo's gateway.

The agent calling us may want any of four formats:
  - OpenAI Chat Completions   (POST /v1/chat/completions, /openai/v1/chat/completions)
  - OpenAI Responses API      (POST /v1/responses, /openai/v1/responses)
  - Anthropic Messages        (POST /v1/messages, /anthropic/v1/messages)
  - Google Gemini             (POST /v1beta/models/.../generateContent,
                               /genai/v1beta/models/...)

This module's job is to:
  1. Extract canonical text + tool-call payload from each recorded gen_ai span.
  2. Re-emit it in the format the calling route expects.

Tool calls round-trip (LiteLLM copies the OpenAI tool_calls shape verbatim into
gen_ai.output.messages); text content always round-trips. Streaming is not
modeled — one whole response per call.

Mount the trace file at /data/traces.jsonl (override with REPLAY_TRACES).
"""

from __future__ import annotations

from flask import Flask, jsonify
import json
import os
import sys
import time
import uuid
from typing import Any

app = Flask(__name__)


# ── OTLP parsing helpers ───────────────────────────────────────────────

# opentelemetry.proto.trace.v1.Status.StatusCode: 0=UNSET, 1=OK, 2=ERROR.
_STATUS_ERROR = 2


def _attr_map(span: dict[str, Any]) -> dict[str, Any]:
    """Flatten a span's [{key, value:{<type>Value: ...}}] list into a dict.

    OTLP encodes attribute values as type-tagged objects; intValue is a
    *string*-encoded int64, doubleValue a number, boolValue a bool.
    """
    out: dict[str, Any] = {}
    for a in span.get("attributes", []) or []:
        key = a.get("key")
        if key is None:
            continue
        v = a.get("value", {}) or {}
        if "stringValue" in v:
            out[key] = v["stringValue"]
        elif "intValue" in v:
            try:
                out[key] = int(v["intValue"])
            except (TypeError, ValueError):
                out[key] = v["intValue"]
        elif "doubleValue" in v:
            out[key] = v["doubleValue"]
        elif "boolValue" in v:
            out[key] = v["boolValue"]
    return out


def _iter_spans(req: dict[str, Any]):
    """Flatten one ExportTraceServiceRequest into its spans."""
    for rs in req.get("resourceSpans", []) or []:
        for ss in rs.get("scopeSpans", []) or []:
            for span in ss.get("spans", []) or []:
                yield span


# ── Canonicalization: OTel gen_ai span → canonical turn ────────────────
# Canonical turn: {"text", "tool_calls", "finish_reason", "model"}.
# The emitters below map that into whatever wire format the route expects.


def _canonicalize_span(attrs: dict[str, Any]) -> dict[str, Any]:
    """Extract canonical text + tool_calls from a gen_ai span's attributes.

    `gen_ai.output.messages` is a JSON *string* (OTEL GenAI 1.38 shape):
        [{"role": "assistant",
          "parts": [{"type": "text", "content": "..."}],
          "tool_calls": [{"id","type","function":{"name","arguments"}}],
          "finish_reason": "stop"}]
    """
    text_parts: list[str] = []
    tool_calls: list[dict[str, Any]] = []
    finish_reason = "stop"

    raw = attrs.get("gen_ai.output.messages")
    messages: Any = []
    if isinstance(raw, str) and raw:
        try:
            messages = json.loads(raw)
        except json.JSONDecodeError:
            messages = []
    elif isinstance(raw, list):
        messages = raw

    for msg in messages:
        if not isinstance(msg, dict):
            continue
        for part in msg.get("parts", []) or []:
            if (
                isinstance(part, dict)
                and part.get("type") == "text"
                and part.get("content")
            ):
                text_parts.append(part["content"])
        for tc in msg.get("tool_calls", []) or []:
            tool_calls.append(tc)  # already OpenAI {id,type,function:{name,arguments}}
        if msg.get("finish_reason"):
            finish_reason = msg["finish_reason"]

    model = (
        attrs.get("gen_ai.response.model")
        or attrs.get("gen_ai.request.model")
        or "replay"
    )
    return {
        "text": "".join(text_parts),
        "tool_calls": tool_calls,
        "finish_reason": finish_reason,
        "model": model,
    }


# ── Trace loading ──────────────────────────────────────────────────────


def _load_turns(path: str) -> list[dict[str, Any]]:
    """Parse an OTLP/JSON trace file into ordered canonical turns.

    Only spans that recorded an LLM response (carry gen_ai.output.messages) and
    did not error are replayable. Spans are ordered by startTimeUnixNano so the
    turns play back in the sequence the agent originally made them — even when
    the collector batched them across lines or out of order.
    """
    collected: list[tuple[int, dict[str, Any]]] = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                req = json.loads(line)
            except json.JSONDecodeError:
                continue
            for span in _iter_spans(req):
                attrs = _attr_map(span)
                if "gen_ai.output.messages" not in attrs:
                    continue
                if (span.get("status", {}) or {}).get("code") == _STATUS_ERROR:
                    continue
                try:
                    start = int(span.get("startTimeUnixNano", "0"))
                except (TypeError, ValueError):
                    start = 0
                collected.append((start, _canonicalize_span(attrs)))
    collected.sort(key=lambda t: t[0])
    return [canon for _, canon in collected]


traces_path = os.environ.get("REPLAY_TRACES", "/data/traces.jsonl")

if os.path.exists(traces_path):
    turns: list[dict[str, Any]] = _load_turns(traces_path)
    print(
        f"[replay] loaded {len(turns)} gen_ai turns from {traces_path}",
        file=sys.stderr,
    )
else:
    turns = []
    print(f"[replay] WARNING: no trace file at {traces_path}", file=sys.stderr)

call_index = 0


# ── Emitters: canonical → wire format per route ────────────────────────


def _emit_chat_completions(canon: dict[str, Any]) -> dict[str, Any]:
    msg: dict[str, Any] = {"role": "assistant", "content": canon["text"] or None}
    if canon["tool_calls"]:
        msg["tool_calls"] = canon["tool_calls"]
    return {
        "id": f"chatcmpl-{uuid.uuid4().hex[:24]}",
        "object": "chat.completion",
        "created": int(time.time()),
        "model": canon.get("model", "replay"),
        "choices": [
            {
                "index": 0,
                "message": msg,
                "finish_reason": canon["finish_reason"],
            }
        ],
    }


def _emit_responses_api(canon: dict[str, Any]) -> dict[str, Any]:
    output: list[dict[str, Any]] = []
    if canon["text"]:
        output.append(
            {
                "id": f"msg_{uuid.uuid4().hex[:24]}",
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": [{"type": "output_text", "text": canon["text"]}],
            }
        )
    for tc in canon["tool_calls"]:
        fn = tc.get("function", {})
        output.append(
            {
                "id": tc.get("id") or f"call_{uuid.uuid4().hex[:8]}",
                "type": "function_call",
                "call_id": tc.get("id"),
                "name": fn.get("name", ""),
                "arguments": fn.get("arguments", "{}"),
            }
        )
    return {
        "id": f"resp_{uuid.uuid4().hex[:24]}",
        "object": "response",
        "created_at": int(time.time()),
        "status": "completed",
        "model": canon.get("model", "replay"),
        "output": output,
    }


def _emit_anthropic_messages(canon: dict[str, Any]) -> dict[str, Any]:
    content: list[dict[str, Any]] = []
    if canon["text"]:
        content.append({"type": "text", "text": canon["text"]})
    for tc in canon["tool_calls"]:
        fn = tc.get("function", {})
        args = fn.get("arguments", "{}")
        try:
            args_obj = json.loads(args) if isinstance(args, str) else args
        except json.JSONDecodeError:
            args_obj = {"_raw": args}
        content.append(
            {
                "type": "tool_use",
                "id": tc.get("id") or f"toolu_{uuid.uuid4().hex[:24]}",
                "name": fn.get("name", ""),
                "input": args_obj,
            }
        )
    stop_reason = {
        "stop": "end_turn",
        "length": "max_tokens",
        "tool_calls": "tool_use",
    }.get(canon["finish_reason"], "end_turn")
    return {
        "id": f"msg_{uuid.uuid4().hex[:24]}",
        "type": "message",
        "role": "assistant",
        "model": canon.get("model", "replay"),
        "content": content,
        "stop_reason": stop_reason,
        "usage": {"input_tokens": 0, "output_tokens": 0},
    }


def _emit_gemini(canon: dict[str, Any]) -> dict[str, Any]:
    parts: list[dict[str, Any]] = []
    if canon["text"]:
        parts.append({"text": canon["text"]})
    for tc in canon["tool_calls"]:
        fn = tc.get("function", {})
        args = fn.get("arguments", "{}")
        try:
            args_obj = json.loads(args) if isinstance(args, str) else args
        except json.JSONDecodeError:
            args_obj = {"_raw": args}
        parts.append({"functionCall": {"name": fn.get("name", ""), "args": args_obj}})
    return {
        "candidates": [
            {
                "content": {"parts": parts, "role": "model"},
                "finishReason": "STOP",
                "index": 0,
            }
        ],
        "modelVersion": canon.get("model", "replay"),
    }


# ── The serve-next-response engine ─────────────────────────────────────


def _next(emit_fn) -> Any:
    """Pull the next recorded turn and emit it in the requested format. On
    exhaustion serve a benign empty-ish response so the agent doesn't blow up
    on a missing reply.
    """
    global call_index
    if call_index < len(turns):
        canon = turns[call_index]
        call_index += 1
        print(
            f"[replay] {call_index}/{len(turns)}: {len(canon['text'])} chars, "
            f"{len(canon['tool_calls'])} tool_calls → {emit_fn.__name__}",
            file=sys.stderr,
        )
    else:
        print(
            f"[replay] EXHAUSTED after {call_index} (returning empty in target format)",
            file=sys.stderr,
        )
        canon = {
            "text": "REPLAY_EXHAUSTED",
            "tool_calls": [],
            "finish_reason": "stop",
            "model": "replay",
        }
    return jsonify(emit_fn(canon))


# ── Routes ─────────────────────────────────────────────────────────────


@app.get("/health")
def health():
    return "ok"


@app.route("/v1/chat/completions", methods=["POST"])
@app.route("/openai/v1/chat/completions", methods=["POST"])
def chat_completions():
    return _next(_emit_chat_completions)


@app.route("/v1/messages", methods=["POST"])
@app.route("/anthropic/v1/messages", methods=["POST"])
def messages():
    return _next(_emit_anthropic_messages)


@app.route("/v1/responses", methods=["POST"])
@app.route("/openai/v1/responses", methods=["POST"])
def responses_api():
    return _next(_emit_responses_api)


@app.route("/v1beta/models/<path:model>", methods=["POST"])
@app.route("/genai/v1beta/models/<path:model>", methods=["POST"])
def gemini_generate(model):
    return _next(_emit_gemini)


@app.route("/", methods=["HEAD", "GET"])
def root():
    return "ok"


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "4000"))
    app.run(host="0.0.0.0", port=port)
