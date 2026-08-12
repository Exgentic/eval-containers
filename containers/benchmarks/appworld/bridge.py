#!/usr/bin/env python3
# Root-owned code-execution bridge for the AppWorld runtime. Runs in the
# background from /entrypoint.sh, before the agent (uid 1002) starts.
#
# Exposes exactly two HTTP routes on 127.0.0.1: GET /health and POST
# /execute. Deliberately does NOT expose evaluate() over HTTP — failing
# AppWorld assertions embed the literal expected values in their message, so
# a reachable /evaluate route would let the agent read the answer, and the
# agent shares this container's network namespace so any bound TCP port is
# reachable to it regardless of uid.
#
# AppWorld() re-initializes by deleting its own output directory (see
# _prepare_directories in appworld/environment.py) — opening a fresh session
# per call, as an earlier version of this file did, silently discards every
# prior call's state. The correct usage (matching appworld's own cli.py) is
# one long-lived session for the whole task: execute() already persists to
# disk after every call internally, so this process just needs to keep that
# one `world` object alive and call execute() on it repeatedly.
#
# Grading reuses that same live object instead of opening a second one, and
# is triggered by SIGUSR1 rather than an HTTP route: signals respect uid
# (only root can signal a root-owned process), so /grade.sh (root, run after
# the agent's own process has exited) can ask for evaluation without the
# agent ever having a network path to it.

import json
import os
import signal
import sys
import traceback
from http.server import BaseHTTPRequestHandler, HTTPServer

from appworld import AppWorld

TASK_ID = os.environ["APPWORLD_TASK_ID"]
EXPERIMENT_NAME = "agent"
PORT = int(os.environ.get("APPWORLD_BRIDGE_PORT", "8123"))

world = None


def handle_evaluate_signal(signum, frame):
    try:
        result = world.evaluate().to_dict()
    except Exception as e:
        result = {"success": False, "error": f"{type(e).__name__}: {e}"}

    os.makedirs("/logs/appworld", exist_ok=True)
    os.makedirs("/logs/verifier", exist_ok=True)
    with open("/logs/appworld/evaluation.json", "w") as f:
        json.dump(result, f, indent=2, default=str)
    reward = 1.0 if result.get("success") else 0.0
    with open("/logs/verifier/reward.txt", "w") as f:
        f.write(str(reward))
    sys.exit(0)


class Handler(BaseHTTPRequestHandler):
    def _send_json(self, status, payload):
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            self._send_json(200, {"status": "ok"})
        else:
            self._send_json(404, {"error": "not found"})

    def do_POST(self):
        if self.path != "/execute":
            self._send_json(404, {"error": "not found"})
            return
        length = int(self.headers.get("Content-Length", 0))
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
            code = body["code"]
        except Exception:
            self._send_json(400, {"error": 'expected JSON body: {"code": "..."}'})
            return
        try:
            output = world.execute(code)
            self._send_json(200, {"output": output})
        except Exception as e:
            self._send_json(200, {"error": f"{type(e).__name__}: {e}"})

    def log_message(self, format, *args):
        pass


def main():
    global world
    signal.signal(signal.SIGUSR1, handle_evaluate_signal)

    # Opened once, kept alive for the process lifetime (see module docstring
    # for why re-opening per call is wrong). First open is 4-5s, so /health
    # only reports ready once this has actually completed.
    try:
        world = AppWorld(task_id=TASK_ID, experiment_name=EXPERIMENT_NAME)
    except Exception:
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)

    # Single-threaded: AppWorld's sqlite connections are bound to the thread
    # that created them, and a threading server would call world.execute()
    # from a different thread than the one that opened `world` above. One
    # agent, sequential turns, doesn't need concurrency.
    server = HTTPServer(("127.0.0.1", PORT), Handler)
    server.serve_forever()


if __name__ == "__main__":
    main()
