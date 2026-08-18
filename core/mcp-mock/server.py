#!/usr/bin/env python3
"""mcp-mock — minimal MCP server for the agent MCP smoke suite.

The MCP analogue of `models/replay`: a stand-in server that agents can
register and connect to offline, whose only job is to *observe* what the
agent did and say so on stderr. It serves a fixed two-tool list and never
talks to a model.

Why observe `tools/list` rather than `tools/call`: the list request is
part of the MCP initialization handshake, so it fires as soon as the
agent connects — before, and independently of, any decision the model
makes. That keeps the smoke test free of inference. Asserting on
`tools/call` instead would require the model to actually choose to use a
tool, which needs a real (or carefully fixtured) completion and turns a
connectivity test into a capability test.

Transport is streamable HTTP (MCP's current remote transport; SSE is
deprecated upstream and every CLI in the fleet that supports remote MCP
supports streamable HTTP). Stdlib only — no flask, no fastmcp, no `mcp`
package — so the image is python:3.12-slim plus this file.

Protocol surface, per the spec's Streamable HTTP transport:
  POST   /mcp     JSON-RPC in, JSON-RPC out (single or batch)
  GET    /mcp     405 — we offer no server-initiated stream. Clients
                  treat this as "no stream available" and carry on.
  DELETE /mcp     session teardown
  GET    /health  liveness, for the compose/testcontainers healthcheck

Stderr markers (the test's assertion surface):
  [mcp] listening on :8000 with N tools    — startup, gate on this
  [mcp] initialize <client>                — handshake opened
  [mcp] tools/list -> N tools              — THE pass condition
  [mcp] tools/call <name>                  — bonus, not asserted
"""

import json
import sys
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = 8000

# The protocol version we advertise when a client doesn't name one. We
# echo the client's requested version back when it does send one — the
# spec allows the server to accept a version it supports, and echoing
# keeps us compatible with clients pinned to older revisions rather than
# forcing a negotiation failure in a test fixture.
PROTOCOL_VERSION = "2025-06-18"

# Two trivial, deterministic tools. `eval_add` is the one a task would
# ask for (a verifiable answer the model cannot produce by guessing the
# format); `eval_echo` exists so `tools/list` returns a plural list and a
# client that mishandles single-element arrays gets caught.
TOOLS = [
    {
        "name": "eval_add",
        "description": "Add two integers and return the sum.",
        "inputSchema": {
            "type": "object",
            "properties": {"a": {"type": "integer"}, "b": {"type": "integer"}},
            "required": ["a", "b"],
        },
    },
    {
        "name": "eval_echo",
        "description": "Echo the given text back unchanged.",
        "inputSchema": {
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"],
        },
    },
]


def log(msg):
    """Stderr, unbuffered. The test polls this stream, so a buffered
    write is a hung test — mirrors replay's `[replay] ...` markers."""
    print(f"[mcp] {msg}", file=sys.stderr, flush=True)


def result(req_id, payload):
    return {"jsonrpc": "2.0", "id": req_id, "result": payload}


def error(req_id, code, message):
    return {"jsonrpc": "2.0", "id": req_id, "error": {"code": code, "message": message}}


def call_tool(name, args):
    """Execute a tool. Returns an MCP tool result (content + isError)."""
    if name == "eval_add":
        try:
            text = str(int(args["a"]) + int(args["b"]))
        except (KeyError, TypeError, ValueError) as e:
            return {
                "content": [{"type": "text", "text": f"bad arguments: {e}"}],
                "isError": True,
            }
    elif name == "eval_echo":
        text = str(args.get("text", ""))
    else:
        return {
            "content": [{"type": "text", "text": f"unknown tool: {name}"}],
            "isError": True,
        }
    return {"content": [{"type": "text", "text": text}], "isError": False}


def dispatch(msg):
    """Handle one JSON-RPC message. Returns a response dict, or None for
    notifications (which per JSON-RPC get no reply — the HTTP layer
    answers 202 instead)."""
    method = msg.get("method")
    req_id = msg.get("id")
    params = msg.get("params") or {}

    # Notifications: no id, no response. `notifications/initialized`
    # closes the handshake; clients send it and expect 202, not a body.
    if req_id is None:
        if method:
            log(f"notification {method}")
        return None

    if method == "initialize":
        client = (params.get("clientInfo") or {}).get("name", "unknown")
        log(f"initialize {client}")
        return result(
            req_id,
            {
                "protocolVersion": params.get("protocolVersion", PROTOCOL_VERSION),
                "capabilities": {"tools": {"listChanged": False}},
                "serverInfo": {"name": "mcp-mock", "version": "1.0.0"},
            },
        )

    if method == "tools/list":
        log(f"tools/list -> {len(TOOLS)} tools")
        return result(req_id, {"tools": TOOLS})

    if method == "tools/call":
        name = params.get("name", "")
        log(f"tools/call {name}")
        return result(req_id, call_tool(name, params.get("arguments") or {}))

    # Some clients probe for capabilities we don't advertise. Answering
    # with an empty list is friendlier than -32601: a hard error here can
    # abort an otherwise-healthy client's startup.
    if method in ("resources/list", "prompts/list"):
        log(f"{method} -> empty")
        key = method.split("/")[0]
        return result(req_id, {key: []})

    if method == "ping":
        return result(req_id, {})

    log(f"unhandled method {method}")
    return error(req_id, -32601, f"method not found: {method}")


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    # BaseHTTPRequestHandler logs every request to stderr by default,
    # which would drown the markers the test greps for.
    def log_message(self, fmt, *args):
        pass

    def _send(self, code, body=None, headers=None):
        blob = b"" if body is None else json.dumps(body).encode()
        self.send_response(code)
        if blob:
            self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(blob)))
        for k, v in (headers or {}).items():
            self.send_header(k, v)
        self.end_headers()
        if blob:
            self.wfile.write(blob)

    def do_GET(self):
        if self.path == "/health":
            self._send(200, {"status": "ok", "tools": len(TOOLS)})
            return
        if self.path.startswith("/mcp"):
            # No server-initiated SSE stream. 405 is the spec's way to
            # say so; clients fall back to POST-only and keep working.
            self._send(405, None, {"Allow": "POST, DELETE"})
            return
        self._send(404)

    def do_DELETE(self):
        if self.path.startswith("/mcp"):
            log("session closed")
            self._send(200)
            return
        self._send(404)

    def do_POST(self):
        if not self.path.startswith("/mcp"):
            self._send(404)
            return
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length) if length else b""
        try:
            msg = json.loads(raw)
        except json.JSONDecodeError:
            self._send(
                400,
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32700, "message": "parse error"},
                },
            )
            return

        # A batch is a JSON array; a single call is an object. Both are
        # legal JSON-RPC and different SDKs use different ones.
        batch = isinstance(msg, list)
        msgs = msg if batch else [msg]
        replies = [r for r in (dispatch(m) for m in msgs) if r is not None]

        # Every message was a notification — nothing to reply with. The
        # spec wants 202 Accepted with an empty body here; returning a
        # JSON `null` instead makes strict clients throw.
        if not replies:
            self._send(202)
            return

        # A fresh session id on the initialize response. We don't
        # validate it on later requests (a mock has nothing to protect),
        # but clients that expect the header echo it back and some
        # refuse to proceed without having received one.
        headers = {}
        if any(m.get("method") == "initialize" for m in msgs):
            headers["Mcp-Session-Id"] = uuid.uuid4().hex

        self._send(200, replies if batch else replies[0], headers)


if __name__ == "__main__":
    log(f"listening on :{PORT} with {len(TOOLS)} tools")
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
