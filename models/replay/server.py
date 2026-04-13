"""Replay model: serves recorded LLM responses for deterministic testing.

Reads a trajectory.jsonl file (LiteLLM StandardLoggingPayload format) and
serves the responses in order. From the eval container's perspective, this
is indistinguishable from a real LiteLLM proxy.

Mount the trajectory file at /data/trajectory.jsonl.
"""

from flask import Flask, request, jsonify
import json, sys, os

app = Flask(__name__)

# Load recorded responses from trajectory
responses = []
trajectory_path = os.environ.get("REPLAY_TRAJECTORY", "/data/trajectory.jsonl")

if os.path.exists(trajectory_path):
    with open(trajectory_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            entry = json.loads(line)
            # Extract the response from the StandardLoggingPayload
            resp = entry.get("response", entry.get("result"))
            if resp:
                if isinstance(resp, str):
                    resp = json.loads(resp)
                responses.append(resp)
    print(f"[replay] loaded {len(responses)} responses from {trajectory_path}", file=sys.stderr)
else:
    print(f"[replay] WARNING: no trajectory at {trajectory_path}", file=sys.stderr)

call_index = 0

@app.get("/health")
def health():
    return "ok"

@app.route("/v1/chat/completions", methods=["POST"])
def chat_completions():
    global call_index
    if call_index < len(responses):
        resp = responses[call_index]
        call_index += 1
        print(f"[replay] serving response {call_index}/{len(responses)}", file=sys.stderr)
        return jsonify(resp)
    else:
        print(f"[replay] WARNING: no more recorded responses (got {call_index} calls)", file=sys.stderr)
        return jsonify({
            "id": "replay-exhausted",
            "object": "chat.completion",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "REPLAY_EXHAUSTED"}, "finish_reason": "stop"}]
        })

@app.route("/v1/messages", methods=["POST"])
def messages():
    """Anthropic-format endpoint — same replay logic."""
    global call_index
    if call_index < len(responses):
        resp = responses[call_index]
        call_index += 1
        print(f"[replay] serving response {call_index}/{len(responses)} (messages)", file=sys.stderr)
        return jsonify(resp)
    else:
        print(f"[replay] WARNING: no more recorded responses", file=sys.stderr)
        return jsonify({
            "id": "replay-exhausted",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "REPLAY_EXHAUSTED"}],
            "stop_reason": "end_turn"
        })

# Catch-all for HEAD / and other probes
@app.route("/", methods=["HEAD", "GET"])
def root():
    return "ok"

if __name__ == "__main__":
    port = int(os.environ.get("PORT", "4000"))
    app.run(host="0.0.0.0", port=port)
