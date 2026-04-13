"""LLM Bridge: a fake OpenAI API that lets a real agent handle tool-calling benchmarks.

Benchmarks (TAU-bench, BFCL) call this as their "LLM". Instead of forwarding to a model,
the bridge queues the request for an agent to pick up, decide, and respond.

Agent polls GET /next, gets the chat completion request (with tools).
Agent posts POST /respond with a chat completion response (with tool_calls or content).
Benchmark sees a normal OpenAI API response.

Start: python bridge.py [--port 8000]
"""

from flask import Flask, request, jsonify
import queue, sys, json, logging

app = Flask(__name__)
logging.basicConfig(level=logging.INFO, format="%(message)s")

inbox = queue.Queue()   # benchmark → agent
outbox = queue.Queue()  # agent → benchmark

@app.post("/v1/chat/completions")
def chat_completions():
    """Benchmark calls this thinking it's an LLM."""
    data = request.json
    app.logger.info(f"← benchmark: {len(data.get('messages',[]))} messages, {len(data.get('tools',[]))} tools")
    inbox.put(data)
    response = outbox.get()  # blocks until agent responds
    app.logger.info(f"→ benchmark: {json.dumps(response)[:200]}")
    return jsonify(response)

@app.get("/next")
def next_request():
    """Agent polls this to get the next request."""
    data = inbox.get()  # blocks until benchmark sends
    return jsonify(data)

@app.post("/respond")
def respond():
    """Agent posts its response here."""
    outbox.put(request.json)
    return "ok"

@app.get("/health")
def health():
    return "ok"

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
    app.run(host="0.0.0.0", port=port)
