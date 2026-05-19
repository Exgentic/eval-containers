"""Replay model: serves recorded LLM responses for deterministic testing.

Reads a trajectory.jsonl file (LiteLLM StandardLoggingPayload format) and
serves the responses in order. From the eval container's perspective, this
is indistinguishable from a real LiteLLM proxy.

The trajectory file may contain responses in any of three formats:
  - OpenAI Chat Completions (response.choices[].message.content)
  - OpenAI Responses API   (response.output[].content[].text)
  - Anthropic Messages      (response.content[].text)

The agent calling us may want any of four formats:
  - OpenAI Chat Completions   (POST /v1/chat/completions, /openai/v1/chat/completions)
  - OpenAI Responses API      (POST /v1/responses, /openai/v1/responses)
  - Anthropic Messages        (POST /v1/messages, /anthropic/v1/messages)
  - Google Gemini             (POST /v1beta/models/.../generateContent,
                               /genai/v1beta/models/...)

This module's job is to:
  1. Extract the text + tool-call payload from the recorded response (whatever
     format it was originally in).
  2. Re-emit it in the format the calling route expects.

This lets us replay any agent against any recorded fixture. Tool calls and
streaming are best-effort: text content always round-trips; tool_calls are
forwarded when the recorded format had them and the target format supports
them.

Mount the trajectory file at /data/trajectory.jsonl.
"""

from __future__ import annotations

from flask import Flask, request, jsonify
import json
import os
import sys
import time
import uuid
from typing import Any

app = Flask(__name__)


# ── Trajectory loading ─────────────────────────────────────────────────

responses: list[dict[str, Any]] = []
trajectory_path = os.environ.get("REPLAY_TRAJECTORY", "/data/trajectory.jsonl")

if os.path.exists(trajectory_path):
    with open(trajectory_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            entry = json.loads(line)
            if entry.get("status") == "failure":
                continue
            resp = entry.get("response")
            if resp:
                if isinstance(resp, str):
                    resp = json.loads(resp)
                responses.append(resp)
    print(
        f"[replay] loaded {len(responses)} responses from {trajectory_path}",
        file=sys.stderr,
    )
else:
    print(f"[replay] WARNING: no trajectory at {trajectory_path}", file=sys.stderr)

call_index = 0


# ── Canonicalization ──────────────────────────────────────────────────
# Each recorded response gets normalized into a small dict:
#   {"text": "...", "tool_calls": [...], "finish_reason": "stop"}
# Translation then maps that into whatever format the route expects.


def _canonicalize(resp: dict[str, Any]) -> dict[str, Any]:
    """Extract canonical text + tool_calls from any of the three recorded formats."""
    text_parts: list[str] = []
    tool_calls: list[dict[str, Any]] = []

    # OpenAI Chat Completions
    if "choices" in resp and isinstance(resp["choices"], list):
        for ch in resp["choices"]:
            msg = ch.get("message") or {}
            if isinstance(msg.get("content"), str) and msg["content"]:
                text_parts.append(msg["content"])
            elif isinstance(msg.get("content"), list):
                for part in msg["content"]:
                    if isinstance(part, dict) and part.get("type") == "text":
                        text_parts.append(part.get("text", ""))
            for tc in msg.get("tool_calls", []) or []:
                tool_calls.append(tc)

    # OpenAI Responses API
    if "output" in resp and isinstance(resp["output"], list):
        for item in resp["output"]:
            if not isinstance(item, dict):
                continue
            content = item.get("content")
            if isinstance(content, list):
                for part in content:
                    if not isinstance(part, dict):
                        continue
                    t = part.get("text") or part.get("output_text")
                    if t:
                        text_parts.append(t)
            if item.get("type") == "function_call":
                tool_calls.append(
                    {
                        "id": item.get("call_id") or item.get("id") or f"call_{uuid.uuid4().hex[:8]}",
                        "type": "function",
                        "function": {
                            "name": item.get("name", ""),
                            "arguments": item.get("arguments", "{}"),
                        },
                    }
                )

    # Anthropic Messages
    if "content" in resp and isinstance(resp["content"], list):
        for part in resp["content"]:
            if not isinstance(part, dict):
                continue
            if part.get("type") == "text":
                text_parts.append(part.get("text", ""))
            elif part.get("type") == "tool_use":
                tool_calls.append(
                    {
                        "id": part.get("id") or f"call_{uuid.uuid4().hex[:8]}",
                        "type": "function",
                        "function": {
                            "name": part.get("name", ""),
                            "arguments": json.dumps(part.get("input", {})),
                        },
                    }
                )

    return {
        "text": "".join(text_parts),
        "tool_calls": tool_calls,
        "finish_reason": _extract_finish_reason(resp),
        "model": resp.get("model", "replay"),
    }


def _extract_finish_reason(resp: dict[str, Any]) -> str:
    """Best-effort extraction of finish_reason from any of the three formats."""
    if "choices" in resp and resp["choices"]:
        fr = resp["choices"][0].get("finish_reason")
        if fr:
            return fr
    if "stop_reason" in resp:
        return {
            "end_turn": "stop",
            "max_tokens": "length",
            "tool_use": "tool_calls",
        }.get(resp["stop_reason"], "stop")
    return "stop"


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
    """Pull the next recorded response, canonicalize it, emit in the
    requested format. On exhaustion serve a benign empty-ish response so
    the agent doesn't blow up on a missing reply.
    """
    global call_index
    if call_index < len(responses):
        resp = responses[call_index]
        call_index += 1
        canon = _canonicalize(resp)
        print(
            f"[replay] {call_index}/{len(responses)}: {len(canon['text'])} chars, "
            f"{len(canon['tool_calls'])} tool_calls → {emit_fn.__name__}",
            file=sys.stderr,
        )
    else:
        print(
            f"[replay] EXHAUSTED after {call_index} (returning empty in target format)",
            file=sys.stderr,
        )
        canon = {"text": "REPLAY_EXHAUSTED", "tool_calls": [], "finish_reason": "stop", "model": "replay"}
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
