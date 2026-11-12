"""Mock model with a fixed context window, for tests/run/context.

Serves Anthropic Messages (claude-code) and OpenAI Responses (codex) on :4000.
A request is counted as its body's bytes / 4 tokens; one whose tokens plus its
max_tokens exceed MOCK_WINDOW is rejected with vLLM's context-length error, as a
real server would. Each answer reports that count as usage, so the agent's own
compaction check sees it.

The mock drives the conversation: while the request offers a shell tool it asks
for `seq 1 2000` (~2.2k tokens of output), so the context grows every turn. A
compaction request (the agent's own summary prompt) is answered with a summary;
the first ordinary request after one is answered with "OK", ending the task.

Every request is logged to stderr as one `[mock] {...}` JSON line, which the
test reads: event is tool, text, compact, final, reject or gave-up.
"""

import json
import os
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

WINDOW = int(os.environ["MOCK_WINDOW"])
COMMAND = "seq 1 2000"
MAX_TURNS = 80  # a safety stop, logged as gave-up, so a test fails rather than hangs
SUMMARY_MARKS = (
    "detailed summary of the conversation",
    "CONTEXT CHECKPOINT COMPACTION",
)
state = {"turns": 0, "compacted": False, "seq": 0}


def log(event, **kw):
    print("[mock] " + json.dumps({"event": event, **kw}), file=sys.stderr, flush=True)


def next_id(prefix):
    state["seq"] += 1
    return f"{prefix}{state['seq']}"


def shell_call(req, wire):
    """(name, arguments) for the request's shell tool, or None if it offers none."""
    for tool in req.get("tools") or []:
        name = tool.get("name")
        if wire == "anthropic" and name == "Bash":
            return name, {"command": COMMAND, "description": "Print numbers"}
        if wire == "responses" and name in ("shell_command", "exec_command", "shell"):
            props = (tool.get("parameters") or {}).get("properties") or {}
            if "cmd" in props:
                return name, {"cmd": COMMAND}
            if (props.get("command") or {}).get("type") == "array":
                return name, {"command": ["bash", "-lc", COMMAND]}
            return name, {"command": COMMAND}
    return None


def sse(events):
    return "".join(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events)


def anthropic(tokens, text, call):
    mid = next_id("msg_")
    blocks, out = [], 0
    events = [
        {
            "type": "message_start",
            "message": {
                "id": mid,
                "type": "message",
                "role": "assistant",
                "model": "mock",
                "content": [],
                "stop_reason": None,
                "stop_sequence": None,
                "usage": {"input_tokens": tokens, "output_tokens": 0},
            },
        }
    ]
    if text:
        blocks.append({"type": "text", "text": text})
        events += [
            {
                "type": "content_block_start",
                "index": 0,
                "content_block": {"type": "text", "text": ""},
            },
            {
                "type": "content_block_delta",
                "index": 0,
                "delta": {"type": "text_delta", "text": text},
            },
            {"type": "content_block_stop", "index": 0},
        ]
        out += len(text) // 4 + 1
    if call:
        name, args = call
        tid = next_id("toolu_")
        blocks.append({"type": "tool_use", "id": tid, "name": name, "input": args})
        i = len(blocks) - 1
        events += [
            {
                "type": "content_block_start",
                "index": i,
                "content_block": {
                    "type": "tool_use",
                    "id": tid,
                    "name": name,
                    "input": {},
                },
            },
            {
                "type": "content_block_delta",
                "index": i,
                "delta": {"type": "input_json_delta", "partial_json": json.dumps(args)},
            },
            {"type": "content_block_stop", "index": i},
        ]
        out += 20
    stop = "tool_use" if call else "end_turn"
    events += [
        {
            "type": "message_delta",
            "delta": {"stop_reason": stop, "stop_sequence": None},
            "usage": {"output_tokens": out},
        },
        {"type": "message_stop"},
    ]
    body = {
        "id": mid,
        "type": "message",
        "role": "assistant",
        "model": "mock",
        "content": blocks,
        "stop_reason": stop,
        "stop_sequence": None,
        "usage": {"input_tokens": tokens, "output_tokens": out},
    }
    return events, body


def responses(tokens, text, call):
    rid = next_id("resp_")
    items, out = [], 0
    if text:
        items.append(
            {
                "type": "message",
                "id": next_id("msg_"),
                "role": "assistant",
                "status": "completed",
                "content": [{"type": "output_text", "text": text, "annotations": []}],
            }
        )
        out += len(text) // 4 + 1
    if call:
        name, args = call
        items.append(
            {
                "type": "function_call",
                "id": next_id("fc_"),
                "call_id": next_id("call_"),
                "name": name,
                "arguments": json.dumps(args),
                "status": "completed",
            }
        )
        out += 20
    usage = {
        "input_tokens": tokens,
        "input_tokens_details": {"cached_tokens": 0},
        "output_tokens": out,
        "output_tokens_details": {"reasoning_tokens": 0},
        "total_tokens": tokens + out,
    }
    body = {
        "id": rid,
        "object": "response",
        "status": "completed",
        "model": "mock",
        "output": items,
        "usage": usage,
    }
    events = [
        {"type": "response.created", "response": {"id": rid, "status": "in_progress"}}
    ]
    for i, item in enumerate(items):
        if item["type"] == "message":
            events.append(
                {"type": "response.output_text.delta", "output_index": i, "delta": text}
            )
        events.append(
            {"type": "response.output_item.done", "output_index": i, "item": item}
        )
    events.append({"type": "response.completed", "response": body})
    return events, body


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, code, body, stream_events=None):
        data = (
            sse(stream_events) if stream_events is not None else json.dumps(body)
        ).encode()
        self.send_response(code)
        self.send_header(
            "Content-Type",
            "text/event-stream" if stream_events is not None else "application/json",
        )
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.reply(200, {"object": "list", "data": [{"id": "mock", "object": "model"}]})

    def do_POST(self):
        raw = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        path = self.path.split("?")[0]
        tokens = len(raw) // 4
        if path.endswith("/messages/count_tokens"):
            return self.reply(200, {"input_tokens": tokens})
        if path.endswith("/v1/messages"):
            wire, build = "anthropic", anthropic
        elif path.endswith("/v1/responses"):
            wire, build = "responses", responses
        else:
            log("unknown-path", path=path)
            return self.reply(404, {"error": {"message": f"no route {path}"}})
        req = json.loads(raw or b"{}")
        cap = req.get("max_tokens") or req.get("max_output_tokens") or 0
        if tokens + cap > WINDOW:
            log("reject", tokens=tokens, max_tokens=cap)
            msg = (
                f"This model's maximum context length is {WINDOW} tokens. However, you requested "
                f"{tokens + cap} tokens ({tokens} in the messages, {cap} in the completion). "
                "Please reduce the length of the messages or completion."
            )
            return self.reply(
                400,
                {
                    "object": "error",
                    "type": "BadRequestError",
                    "message": msg,
                    "error": {"message": msg, "type": "BadRequestError", "code": 400},
                },
            )
        text = json.dumps(req)
        call = None
        if any(m in text for m in SUMMARY_MARKS):
            state["compacted"] = True
            log("compact", tokens=tokens, max_tokens=cap)
            reply = "<summary>The task asks for a reply of OK. Numbers were printed several times.</summary>"
        elif (found := shell_call(req, wire)) is None:
            log("text", tokens=tokens, max_tokens=cap)
            reply = "OK"
        elif state["compacted"]:
            log("final", tokens=tokens, max_tokens=cap)
            reply = "OK"
        elif state["turns"] >= MAX_TURNS:
            log("gave-up", tokens=tokens, max_tokens=cap)
            reply = "OK"
        else:
            state["turns"] += 1
            log("tool", tokens=tokens, max_tokens=cap, turn=state["turns"])
            reply, call = "", found
        events, body = build(tokens, reply, call)
        self.reply(200, body, events if req.get("stream") else None)


if __name__ == "__main__":
    server = ThreadingHTTPServer(("0.0.0.0", 4000), Handler)
    log("listening", window=WINDOW)
    server.serve_forever()
