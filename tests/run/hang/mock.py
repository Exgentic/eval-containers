"""Mock model for tests/run/hang: one shell command that never returns, then a final answer.

Serves Anthropic Messages (claude-code) and OpenAI Chat Completions (pi) on
:4000. The first request that offers a shell tool is answered with a call to a
command that sleeps for ~11 days, with no timeout of its own, the way a model
starts a watch-mode test runner or a deadlocked script. It is a Python sleep, not
`sleep N`: Claude Code refuses a standalone `sleep` before running it. A request
that carries that call's result is answered with the final text, ending the
task. Any other request (a title or summary side call) gets "OK".

MOCK_MODE picks a different model. `budget` answers every request the way
LiteLLM (v1.89.1, core/litellm) answers a key whose budget is spent: HTTP 429,
`budget_exceeded`, no Retry-After, an error that never clears. `overflow`
answers the first request that offers a shell tool with a short command, so
there is a conversation to compact, then rejects every later one as too long,
with LiteLLM's context-window error, until the agent has asked for a summary of
the conversation (a compaction, answered in opencode's template). After that it
answers with the final text, so an agent that compacts finishes and one that
only retries never does.

Each request is logged to stderr as one `[mock] {...}` JSON line, which the test
reads: event is tool, final, text, summary, budget or overflow (and listening at start,
unknown-path for a route it doesn't serve). `tool` carries the description the request gives the
shell tool's timeout; `final` the seconds since the tool call and the start of
the tool's output.
"""

import json
import os
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

COMMAND = "python3 -c 'import time; time.sleep(10**6)'"
FINAL = "DONE"
MODE = os.environ.get("MOCK_MODE", "command")
BUDGET = "Budget has been exceeded! Current cost: 200.09, Max budget: 200.0"
OVERFLOW = (
    "This model's maximum context length is 32768 tokens. However, you requested "
    "40000 tokens (36000 in the messages, 4000 in the completion). Please reduce "
    "the length of the messages or completion."
)
# LiteLLM's answer when vLLM rejects a call as too long, as agents see it.
OVERFLOW_ERROR = {
    "message": "litellm.ContextWindowExceededError: " + OVERFLOW,
    "type": None,
    "param": None,
    "code": "400",
}
SUMMARY = "## Objective\nReply DONE.\n\n## Work State\nA command printed numbers; nothing is left to do.\n"
FILL = "seq 1 200"  # overflow mode: a turn before the rejection, so there is a
TURNS = 1  # conversation to compact (opencode: "Nothing to compact yet" without)
state = {"tool_at": None, "seq": 0, "turns": 0, "summarized": False}
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


def timeout_doc(req, wire):
    """The description the request gives its shell tool's timeout parameter, or ""."""
    for t in req.get("tools") or []:
        f = t if wire == "anthropic" else (t.get("function") or {})
        if f.get("name") in ("Bash", "bash", "shell"):
            schema = (
                f.get("input_schema" if wire == "anthropic" else "parameters") or {}
            )
            return ((schema.get("properties") or {}).get("timeout") or {}).get(
                "description", ""
            )
    return ""


def shell_tool(req, wire):
    """The name of the request's shell tool, or None if it offers none."""
    for t in req.get("tools") or []:
        name = (
            t.get("name")
            if wire == "anthropic"
            else (t.get("function") or {}).get("name")
        )
        if name in ("Bash", "bash", "shell"):
            return name
    return None


def anthropic(text, call, command=COMMAND):
    blocks, events = [], []
    if text:
        blocks.append({"type": "text", "text": text})
    if call:
        blocks.append(
            {
                "type": "tool_use",
                "id": next_id("toolu_"),
                "name": call,
                "input": {"command": command},
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


def chat(text, call, command=COMMAND):
    msg = {"role": "assistant", "content": text or None}
    if call:
        msg["tool_calls"] = [
            {
                "id": next_id("call_"),
                "type": "function",
                "function": {
                    "name": call,
                    "arguments": json.dumps({"command": command}),
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
        if MODE == "budget":
            # `agent`: the call offers the shell tool, so it is the agent's own
            # turn, not a title or summary side call with its own retries.
            log("budget", agent=tool is not None)
            error = {
                "message": BUDGET,
                "type": "budget_exceeded",
                "param": None,
                "code": "429",
            }
            return self.reply(429, {"error": error})
        with lock:
            summary = "summarize the conversation" in json.dumps(
                req.get("messages") or []
            )
            if summary:
                state["summarized"] = True
                log("summary")
                text = SUMMARY
            elif MODE == "overflow" and tool and state["turns"] < TURNS:
                state["turns"] += 1
                log("tool", command=FILL, turn=state["turns"])
                body, sse = build("", tool, FILL)
                return self.reply(200, body, sse if req.get("stream") else None)
            elif MODE == "overflow" and tool and not state["summarized"]:
                # Rejected until the agent has compacted: a provider keeps
                # rejecting a context that is still too long.
                log("overflow")
                return self.reply(400, {"error": OVERFLOW_ERROR})
            elif MODE == "overflow" and tool:
                log("final", seconds=None, output="")
                text = FINAL
            elif tool and result is not None:
                at = state["tool_at"]
                waited = round(time.time() - at, 1) if at else None
                log("final", seconds=waited, output=result[:300])
                text = FINAL
            elif tool and state["tool_at"] is None:
                state["tool_at"] = time.time()
                log("tool", command=COMMAND, doc=timeout_doc(req, wire))
                text, call = "", tool
            else:
                log("text")
                text = "OK"
        body, sse = build(text, call)
        self.reply(200, body, sse if req.get("stream") else None)


if __name__ == "__main__":
    server = ThreadingHTTPServer(("0.0.0.0", 4000), Handler)
    log("listening", mode=MODE)
    server.serve_forever()
