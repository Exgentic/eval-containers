"""The only desktop the agent can reach: look, and move the mouse.

The control server does far more than an agent is supposed to have — /execute
runs arbitrary commands in the session, which is how a shell-equipped agent can
write a task's expected files instead of doing the work on screen, and score
full marks for it. Upstream never has to think about this: their agent is a
model loop outside the VM emitting pyautogui, not a process with a shell.

So the real server listens on a unix socket the agent's uid cannot reach (setup
and grading open it as root), and this is what listens on the port: screenshot
in, pyautogui out, nothing else. The agent shares the pod's network namespace,
so being unreachable is the only guarantee that holds — not being undocumented.

`/act` takes pyautogui source, which is Python source: concatenating it into a
script and running it is `/execute` with a prefix. So the action is parsed and
every call checked against the allowlist below before anything runs.
"""

import ast
import json
import os
import socket
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SOCKET = os.environ.get("DESKTOP_SOCKET", "/run/desktop/desktop.sock")
PORT = int(os.environ.get("DESKTOP_AGENT_PORT", "5000"))

ALLOWED = {
    "pyautogui": {
        "click",
        "doubleClick",
        "rightClick",
        "middleClick",
        "moveTo",
        "moveRel",
        "dragTo",
        "dragRel",
        "scroll",
        "hscroll",
        "mouseDown",
        "mouseUp",
        "write",
        "typewrite",
        "press",
        "keyDown",
        "keyUp",
        "hotkey",
        "screenshot",
    },
    "time": {"sleep"},
}


def checked(action: str) -> str:
    """The action, or a ValueError naming what is not allowed.

    Only `pyautogui.<f>(…)` and `time.sleep(…)` on literal arguments: no import,
    no assignment, no name lookup, no attribute walk off the module. Anything
    richer is a shell — `__import__`, `pyautogui.__class__`, a comprehension
    that calls `open` — and a shell can write the files the evaluator reads.
    """
    for node in ast.parse(action).body:
        if not isinstance(node, ast.Expr) or not isinstance(node.value, ast.Call):
            raise ValueError("only pyautogui calls, one per line")
        call = node.value
        func = call.func
        if (
            not isinstance(func, ast.Attribute)
            or not isinstance(func.value, ast.Name)
            or func.attr not in ALLOWED.get(func.value.id, ())
        ):
            raise ValueError("not an allowed call")
        for arg in [*call.args, *(k.value for k in call.keywords)]:
            # A list of keys is how `press` and `write` take a sequence.
            parts = arg.elts if isinstance(arg, (ast.List, ast.Tuple)) else [arg]
            if not all(isinstance(x, ast.Constant) for x in parts):
                raise ValueError("arguments must be literals")
    return action


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
                raise ValueError("action must be a string")
            checked(action)
        except (ValueError, KeyError, TypeError, SyntaxError) as exc:
            return self._send(
                400, json.dumps({"error": str(exc)}).encode(), "application/json"
            )
        # The prefix upstream's own controller runs every action under.
        script = "import pyautogui, time\npyautogui.FAILSAFE = False\n" + action
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
