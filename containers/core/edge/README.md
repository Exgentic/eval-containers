# edge

A tool to record every LLM call your code makes — request, response, and
timing, verbatim — and optionally pin them all to one model, by running as a
transparent proxy in front of your model provider. It forwards each call on
whichever wire (OpenAI/Anthropic/Gemini-shaped) it arrived on and appends
every exchange to `calls.jsonl`. Full spec:
[`.agents/edge/RULES.md`](../../../.agents/edge/RULES.md).

Stdlib-only Go — a single static binary with no runtime dependency — so it
installs anywhere the Go toolchain does, independent of this repo:

```bash
go install github.com/Exgentic/eval-containers/containers/core/edge@latest
```

That installs a binary named `edge`. (Inside this repo it's also built into
the `ghcr.io/exgentic/core/edge` image — see `Dockerfile` — where the
existing entrypoint contract names the same binary `start`; the two are
independent consumers of the same source.)

## Configuration

Environment variables only, no flags:

| Variable | Required | Default | Meaning |
|---|---|---|---|
| `OPENAI_API_BASE` | yes | — | Upstream URL to forward calls to |
| `OPENAI_API_KEY` | for real calls | — | Upstream credential, attached per-wire and never recorded |
| `EVAL_MODEL` | no | — | Model to pin every outbound call to; unset forwards the caller's own choice |
| `EDGE_MODEL` | no | — | Same as `EVAL_MODEL`, for use outside the eval-containers framework; `EVAL_MODEL` wins if both are set |
| `EDGE_UPSTREAM` | no | — | `gateway` when upstream serves the `/anthropic`, `/openai`, `/genai` namespaced paths; unset when upstream is a bare provider |
| `LISTEN` | no | `:4100` | Address the edge listens on |
| `OUT` | no | `/output/model/calls.jsonl` | Where call records are appended (JSON Lines) |
| `EDGE_MAX_REQUEST_BYTES` | no | 64MiB | Request size the edge will parse to pin the model; larger requests are refused |
| `EDGE_MAX_RECORD_BYTES` | no | 8MiB | Per-exchange bytes kept in a record before truncating |
| `EDGE_MAX_RETRIES` | no | 2 | Transport-failure retries before any byte reaches the caller |

Serves `/anthropic`, `/openai`, `/genai` (namespaced per
[`gateways/RULES.md`](../../../.agents/gateways/RULES.md) rule 5) plus
`/health`; any other path forwards on the `openai` wire. `edge health` exits
0 when `/health` answers on `LISTEN`, for a shell-free readiness probe.
