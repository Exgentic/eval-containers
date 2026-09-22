"""The only desktop the agent can reach: look, and move the mouse.

The control server does far more than an agent is supposed to have — /execute
runs arbitrary commands in the session, which is how a shell-equipped agent can
write a task's expected files instead of doing the work on screen, and score
full marks for it. Upstream never has to think about this: their agent is a
model loop outside the VM emitting pyautogui, not a process with a shell.

So the real server listens on a unix socket that only root can open (setup and
grading use it), and this is what listens on the port: screenshot in, pyautogui
out, nothing else. The agent shares the pod's network namespace, so being
unreachable is the only guarantee that holds — not being undocumented.
"""

import json
import os
import socket
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SOCKET = os.environ.get("DESKTOP_SOCKET", "/run/desktop/desktop.sock")
PORT = int(os.environ.get("DESKTOP_AGENT_PORT", "5000"))


def call(method: str, path: str, body: bytes | None = None) -> tuple[int, bytes]:
    """One request to the real server over its unix socket."""
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(SOCKET)
        head = f"{method} {path} HTTP/1.1\r\nHost: desktop\r\nConnection: close\r\n"
        if body is not None:
            head += (
                f"Content-Type: application/json\r\n" f"Content-Length: {len(body)}\r\n"
            )
        s.sendall(head.encode() + b"\r\n" + (body or b""))
        chunks = []
        while True:
            chunk = s.recv(65536)
            if not chunk:
                break
            chunks.append(chunk)
    raw = b"".join(chunks)
    header, _, payload = raw.partition(b"\r\n\r\n")
    status = int(header.split(b" ")[1]) if header.startswith(b"HTTP/") else 502
    return status, payload


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send(self, status: int, body: bytes, ctype: str) -> None:
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):  # noqa: N802 - BaseHTTPRequestHandler's spelling
        if urllib.parse.urlparse(self.path).path != "/screenshot":
            return self._send(
                403, b'{"error":"only /screenshot and /act"}', "application/json"
            )
        status, body = call("GET", "/screenshot")
        self._send(status, body, "image/png")

    def do_POST(self):  # noqa: N802
        if urllib.parse.urlparse(self.path).path != "/act":
            return self._send(
                403, b'{"error":"only /screenshot and /act"}', "application/json"
            )
        raw = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        try:
            action = json.loads(raw or b"{}")["action"]
            if not isinstance(action, str):
                raise ValueError
        except (ValueError, KeyError, TypeError):
            return self._send(
                400,
                b'{"error":"send {\\"action\\": \\"<pyautogui>\\"}"}',
                "application/json",
            )
        # The action is pyautogui source and only ever runs as pyautogui source:
        # it is placed in the script here, never taken as a command line.
        script = "import pyautogui, time\n" + action
        status, body = call(
            "POST",
            "/execute",
            json.dumps({"command": ["python3", "-c", script]}).encode(),
        )
        self._send(status, body, "application/json")

    def log_message(self, *a):  # quiet: one line per agent action is noise
        return


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()  # noqa: S104
