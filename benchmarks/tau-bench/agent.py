"""Pass-through agent for TAU-bench via LLM bridge.

Polls the bridge for chat completion requests (from tau-bench),
forwards them to the real model proxy, and returns the response.

This makes the real LLM act as the "agent" in tau-bench's framework,
while tau-bench controls the tool execution and user simulation.
"""

import os
import sys
import time
import json
import urllib.request
import urllib.error

BRIDGE_URL = os.environ.get("BRIDGE_URL", "http://bridge:8000")
MODEL_URL = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
API_KEY = os.environ.get("OPENAI_API_KEY", "sk-proxy")


IDLE_EXIT_SECONDS = int(os.environ.get("TAU_BENCH_IDLE_EXIT_SECONDS", "120"))


def poll_next():
    """Poll bridge /next for the next chat completion request.

    Returns None when the bridge has been idle for IDLE_EXIT_SECONDS, which
    signals that the tau-bench runner has finished and the pass-through
    agent should terminate. Without this, agent.py loops forever and the
    outer sweep runner times out at the EVAL_TIMEOUT boundary.
    """
    url = f"{BRIDGE_URL}/next"
    idle_started = None
    while True:
        try:
            req = urllib.request.Request(url)
            with urllib.request.urlopen(req, timeout=30) as resp:
                return json.loads(resp.read())
        except urllib.error.URLError as e:
            now = time.time()
            if idle_started is None:
                idle_started = now
            if now - idle_started > IDLE_EXIT_SECONDS:
                print(
                    f"[agent] bridge idle > {IDLE_EXIT_SECONDS}s, exiting",
                    file=sys.stderr,
                )
                return None
            print(f"[agent] waiting for bridge: {e}", file=sys.stderr)
            time.sleep(2)
        except Exception as e:
            print(f"[agent] error polling: {e}", file=sys.stderr)
            time.sleep(2)


def forward_to_model(request_data):
    """Forward the chat completion request to the real model."""
    url = f"{MODEL_URL}/chat/completions"
    body = json.dumps(request_data).encode()
    req = urllib.request.Request(
        url,
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {API_KEY}",
        },
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        return json.loads(resp.read())


def post_response(response_data):
    """Post the model response back to the bridge."""
    url = f"{BRIDGE_URL}/respond"
    body = json.dumps(response_data).encode()
    req = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        resp.read()


def main():
    print("[agent] starting pass-through agent loop", file=sys.stderr)
    turn = 0
    while True:
        try:
            # Get next request from bridge (blocks until tau-bench sends one)
            print(f"[agent] waiting for turn {turn}...", file=sys.stderr)
            request_data = poll_next()
            if request_data is None:
                # Bridge idle — runner has finished. Exit cleanly.
                return
            print(
                f"[agent] got request: {len(request_data.get('messages', []))} msgs, "
                f"{len(request_data.get('tools', []))} tools",
                file=sys.stderr,
            )

            # Forward to real model
            response = forward_to_model(request_data)
            print(f"[agent] got model response", file=sys.stderr)

            # Post response back to bridge
            post_response(response)
            print(f"[agent] posted response for turn {turn}", file=sys.stderr)
            turn += 1

        except Exception as e:
            print(f"[agent] error: {e}", file=sys.stderr)
            time.sleep(1)


if __name__ == "__main__":
    main()
