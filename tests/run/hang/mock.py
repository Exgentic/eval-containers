"""Mock model for tests/run/hang: one shell command that never returns, then a final answer.

Serves Anthropic Messages (claude-code) and OpenAI Chat Completions (pi) on
:4000. The first request that offers a shell tool is answered with a call to a
command that sleeps for ~11 days, with no timeout of its own, the way a model
starts a watch-mode test runner or a deadlocked script. It is a Python sleep, not
`sleep N`: Claude Code refuses a standalone `sleep` before running it. A request
that carries that call's result is answered with the final text, ending the
task. Any other request (a title or summary side call) gets "OK".

Each request is logged to stderr as one `[mock] {...}` JSON line, which the test
reads: event is tool, final or text (and listening at start, unknown-path for a
route it doesn't serve). `final` carries the seconds since the tool call and the
start of the tool's output.
"""

import json
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

COMMAND = "python3 -c 'import time; time.sleep(10**6)'"
FINAL = "DONE"
state = {"tool_at": None, "seq": 0}
lock = threading.Lock()


def log(event, **kw):
    print("[mock] " + json.dumps({"event": event, **kw}), file=sys.stderr, flush=True)


def next_id(prefix):
    state["seq"] += 1
    return f"{prefix}{state['seq']}"


def text_of(content):
    """A tool result's text, whether a string or a list of blocks."""
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return " ".join(b.get("text", "") for b in content if isinstance(b, dict))
    return ""


def tool_result(req, wire):
    """The shell call's result text, or None if the request carries none."""
    for m in req.get("messages") or []:
        if wire == "chat" and m.get("role") == "tool":
            return text_of(m.get("content"))
        if wire == "anthropic" and isinstance(m.get("content"), list):
            for b in m["content"]:
                if b.get("type") == "tool_result":
                    return text_of(b.get("content"))
    return None


def shell_tool(req, wire):
    """The name of the request's shell tool, or None if it offers none."""
    for t in req.get("tools") or []:
        name = (
            t.get("name")
            if wire == "anthropic"
            else (t.get("function") or {}).get("name")
        )
        if name in ("Bash", "bash"):
            return name
    return None


def anthropic(text, call):
    blocks, events = [], []
    if text:
        blocks.append({"type": "text", "text": text})
    if call:
        blocks.append(
            {
                "type": "tool_use",
                "id": next_id("toolu_"),
                "name": call,
                "input": {"command": COMMAND},
            }
        )
    stop = "tool_use" if call else "end_turn"
    usage = {"input_tokens": 100, "output_tokens": 10}
    events.append(
        {
            "type": "message_start",
            "message": {
                "id": next_id("msg_"),
                "type": "message",
                "role": "assistant",
                "model": "mock",
                "content": [],
                "stop_reason": None,
                "stop_sequence": None,
                "usage": {"input_tokens": 100, "output_tokens": 0},
            },
        }
    )
    for i, b in enumerate(blocks):
        if b["type"] == "text":
            start, delta = (
                {"type": "text", "text": ""},
                {"type": "text_delta", "text": b["text"]},
            )
        else:
            start = {**b, "input": {}}
            delta = {"type": "input_json_delta", "partial_json": json.dumps(b["input"])}
        events += [
            {"type": "content_block_start", "index": i, "content_block": start},
            {"type": "content_block_delta", "index": i, "delta": delta},
            {"type": "content_block_stop", "index": i},
        ]
    events += [
        {
            "type": "message_delta",
            "delta": {"stop_reason": stop, "stop_sequence": None},
            "usage": {"output_tokens": 10},
        },
        {"type": "message_stop"},
    ]
    body = {
        "id": next_id("msg_"),
        "type": "message",
        "role": "assistant",
        "model": "mock",
        "content": blocks,
        "stop_reason": stop,
        "stop_sequence": None,
        "usage": usage,
    }
    sse = "".join(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events)
    return body, sse


def chat(text, call):
    msg = {"role": "assistant", "content": text or None}
    if call:
        msg["tool_calls"] = [
            {
                "id": next_id("call_"),
                "type": "function",
                "function": {
                    "name": call,
                    "arguments": json.dumps({"command": COMMAND}),
                },
            }
        ]
    finish = "tool_calls" if call else "stop"
    usage = {"prompt_tokens": 100, "completion_tokens": 10, "total_tokens": 110}
    cid = next_id("chatcmpl-")
    body = {
        "id": cid,
        "object": "chat.completion",
        "created": 0,
        "model": "mock",
        "choices": [{"index": 0, "message": msg, "finish_reason": finish}],
        "usage": usage,
    }
    delta = {"role": "assistant", "content": text or ""}
    if call:
        delta["tool_calls"] = [{"index": 0, **msg["tool_calls"][0]}]
    chunks = [
        {
            "id": cid,
            "object": "chat.completion.chunk",
            "created": 0,
            "model": "mock",
            "choices": [{"index": 0, "delta": delta, "finish_reason": None}],
        },
        {
            "id": cid,
            "object": "chat.completion.chunk",
            "created": 0,
            "model": "mock",
            "choices": [{"index": 0, "delta": {}, "finish_reason": finish}],
            "usage": usage,
        },
    ]
    sse = "".join(f"data: {json.dumps(c)}\n\n" for c in chunks) + "data: [DONE]\n\n"
    return body, sse


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, code, body, sse=None):
        data = (sse if sse is not None else json.dumps(body)).encode()
        self.send_response(code)
        self.send_header(
            "Content-Type",
            "text/event-stream" if sse is not None else "application/json",
        )
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.reply(200, {"object": "list", "data": [{"id": "mock", "object": "model"}]})

    def do_POST(self):
        raw = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        path = self.path.split("?")[0]
        if path.endswith("/messages/count_tokens"):
            return self.reply(200, {"input_tokens": 100})
        if path.endswith("/v1/messages"):
            wire, build = "anthropic", anthropic
        elif path.endswith("/chat/completions"):
            wire, build = "chat", chat
        else:
            log("unknown-path", path=path)
            return self.reply(404, {"error": {"message": f"no route {path}"}})
        req = json.loads(raw or b"{}")
        tool = shell_tool(req, wire)
        result = tool_result(req, wire)
        call = None
        with lock:
            if tool and result is not None:
                at = state["tool_at"]
                waited = round(time.time() - at, 1) if at else None
                log("final", seconds=waited, output=result[:300])
                text = FINAL
            elif tool and state["tool_at"] is None:
                state["tool_at"] = time.time()
                log("tool", command=COMMAND)
                text, call = "", tool
            else:
                log("text")
                text = "OK"
        body, sse = build(text, call)
        self.reply(200, body, sse if req.get("stream") else None)


if __name__ == "__main__":
    server = ThreadingHTTPServer(("0.0.0.0", 4000), Handler)
    log("listening")
    server.serve_forever()
