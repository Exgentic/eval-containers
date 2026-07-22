#!/usr/bin/env python3
"""Recording mock upstream for the gateway translation tests.

Stands in for the real LLM upstream ($OPENAI_API_BASE). Its only jobs:

  1. Record every request the gateway forwards — method, path, headers,
     and parsed JSON body — as one JSON line in /output/requests.jsonl.
     This is the "target output request" the translation tests assert on.
  2. Return a minimal, protocol-valid response for the path so the
     gateway finishes cleanly (no retries that would duplicate captures).

It is deliberately dumb: no auth, no upstream calls, no streaming. The
tests never look at the response — they read requests.jsonl and check
what protocol/path/tool the gateway *sent here*. Runs on stock
python:3.12-slim via the stdlib http.server (no pip deps), as root, so
writes to the bind-mounted /output succeed under rootless podman.
"""

import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

RECORD_PATH = "/output/requests.jsonl"
_write_lock = threading.Lock()


def _record(method, path, headers, body_bytes):
    try:
        body = json.loads(body_bytes) if body_bytes else None
    except Exception:
        body = {"__unparsed__": body_bytes.decode("utf-8", "replace")}
    entry = {
        "method": method,
        "path": path,
        "headers": {k.lower(): v for k, v in headers.items()},
        "body": body,
    }
    line = json.dumps(entry) + "\n"
    with _write_lock:
        with open(RECORD_PATH, "a") as f:
            f.write(line)
            f.flush()


def _canned_response(path):
    """Minimal valid response matching the native shape of `path`."""
    if path.endswith("/v1/messages"):
        return {
            "id": "msg_mock",
            "type": "message",
            "role": "assistant",
            "model": "mock",
            "content": [{"type": "text", "text": "ok"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 1, "output_tokens": 1},
        }
    if path.endswith("/v1/responses"):
        return {
            "id": "resp_mock",
            "object": "response",
            "model": "mock",
            "status": "completed",
            "output": [
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "ok"}],
                }
            ],
            "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2},
        }
    if "generateContent" in path:
        return {
            "candidates": [
                {
                    "content": {"role": "model", "parts": [{"text": "ok"}]},
                    "finishReason": "STOP",
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 1,
                "candidatesTokenCount": 1,
                "totalTokenCount": 2,
            },
        }
    # Default: OpenAI chat.completion shape.
    return {
        "id": "chatcmpl_mock",
        "object": "chat.completion",
        "model": "mock",
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": "ok"},
                "finish_reason": "stop",
            }
        ],
        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):  # silence access logging
        pass

    def _send_json(self, obj, status=200):
        payload = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_POST(self):
        length = int(self.headers.get("content-length", 0))
        body = self.rfile.read(length) if length else b""
        _record("POST", self.path, self.headers, body)
        self._send_json(_canned_response(self.path))

    def do_GET(self):
        # Health probes / model-list discovery — return something benign.
        if "/models" in self.path:
            self._send_json({"object": "list", "data": []})
        else:
            self._send_json({"status": "ok"})


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", 8080), Handler).serve_forever()
