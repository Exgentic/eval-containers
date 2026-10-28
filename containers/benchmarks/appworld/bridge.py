#!/usr/bin/env python3
# Code-execution bridge for the AppWorld runtime, run as its own dedicated
# uid (`bridge`, distinct from root and from the agent's uid 1002 — see the
# Dockerfile). Runs in the background from /entrypoint.sh, before the agent
# starts.
#
# Exposes exactly two HTTP routes on 127.0.0.1: GET /health and POST
# /execute. Deliberately does NOT expose evaluate() over HTTP — failing
# AppWorld assertions embed the literal expected values in their message, so
# a reachable /evaluate route would let the agent read the answer, and the
# agent shares this container's network namespace so any bound TCP port is
# reachable to it regardless of uid.
#
# Running as `bridge` rather than root matters for more than the signal
# story below: AppWorld's own code-safety guard (SafetyGuard in
# appworld/common/safety_guard.py) blocks *writes* during execute() but
# explicitly permits *reads* via any path — it was designed to stop the
# agent's code from corrupting the sandbox, not to protect a secret
# co-located on the same filesystem. ground_truth/ sits right next to the
# dbs/ this process legitimately needs, so if this process ran as root, the
# agent's execute()-supplied code could read ground_truth straight off disk
# regardless of that guard. Running as `bridge`, with ground_truth/ chowned
# root-only in the Dockerfile, makes the OS itself enforce what AppWorld's
# guard does not — and since that's a real OS permission, not a flag, the
# constructor below opts out of AppWorld's own default eager
# ground_truth read (load_ground_truth=False) so this process never even
# attempts to touch it until grade.sh has explicitly opened it back up.
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
# is triggered by SIGUSR1 rather than an HTTP route: signals respect uid (the
# agent's uid can't signal a process owned by a different uid without
# CAP_KILL), so /grade.sh (root, run after the agent's own process has
# exited) can ask for evaluation without the agent ever being able to
# trigger it itself, over HTTP or otherwise.
#
# grade.sh's own sequencing (kill the agent's timed process, THEN chown
# ground_truth/ to `bridge`, THEN signal) assumes the agent has no process
# left running — but run-agent's timeout only bounds the agent's direct
# child; a deliberately-detached grandchild would survive it and could keep
# POSTing to /execute during the gap between that chown and this signal
# actually being delivered, reading ground_truth through a live world.execute()
# the same way root once could. STOP_FILE closes that regardless of timing:
# grade.sh touches it BEFORE the chown (see Dockerfile), and do_POST checks
# it first. The HTTP server below is single-threaded and fully synchronous
# (one request completes before the next is even accepted), so once the
# file exists no further code ever executes here — and anything already
# mid-request when it was created started (and finished) while
# ground_truth/ was still root-only, before the chown that follows it.

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
STOP_FILE = "/run/bridge.stop_accepting"

world = None


def handle_evaluate_signal(signum, frame):
    try:
        # world was opened with load_ground_truth=False (see main()) — the
        # ground_truth/ directory is root-only until grade.sh chowns it to
        # `bridge` immediately before sending this signal, after touching
        # STOP_FILE. Load it now, into the same live object, rather than
        # reconstructing (which would wipe progress).
        from appworld.ground_truth import GroundTruth

        world.task.ground_truth = GroundTruth.load(task_id=TASK_ID, mode="minimal")
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
    # sys.exit(0) here only raises SystemExit in this thread — AppWorld's
    # construction spins up at least one non-daemon thread that survives it,
    # leaving the process alive (confirmed live: thread count 1->2, /health
    # hangs afterward). os._exit() terminates the process unconditionally,
    # at the OS level, regardless of what other threads are doing.
    os._exit(0)


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
        if os.path.exists(STOP_FILE):
            self._send_json(503, {"error": "grading in progress"})
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
        # load_ground_truth=False: AppWorld's constructor eagerly reads
        # ground_truth/metadata.json by default — but that directory is
        # root-only (this process runs as `bridge`, see the Dockerfile), so
        # the default would crash on startup, not just leak the secret.
        # Loaded explicitly, later, in handle_evaluate_signal.
        world = AppWorld(
            task_id=TASK_ID, experiment_name=EXPERIMENT_NAME, load_ground_truth=False
        )
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
