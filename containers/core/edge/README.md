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
| `EDGE_MAX_TOKENS` | no | 0 (none) | Tokens the run may spend, input + output, before calls are refused |
| `EDGE_MAX_COST_USD` | no | 0 (none) | Dollars the run may spend before calls are refused; needs the prices below |
| `EDGE_PRICE_IN_USD_PER_MTOK` | for a cost cap | — | Input price, USD per million tokens |
| `EDGE_PRICE_OUT_USD_PER_MTOK` | for a cost cap | — | Output price, USD per million tokens |
| `EDGE_ON_LIMIT` | no | `refuse` | What crossing a cap does: `refuse` every later call, or `kill` the container |

## Caps

With `EDGE_MAX_TOKENS` or `EDGE_MAX_COST_USD` set, the edge counts what every
call reports — `prompt_tokens`/`completion_tokens`, `input_tokens`/
`output_tokens` (plus Anthropic's cache tokens, which bill as input),
`promptTokenCount`/`candidatesTokenCount`/`thoughtsTokenCount` — streamed or
not. Once a cap is crossed, every later call is answered `402` with a
`budget_exceeded` body and never reaches the upstream; the call in flight is
not cut. Cost needs prices because the edge must not identify the model behind
the handle. `EDGE_ON_LIMIT=kill` additionally stops the container, which also
ends the grading that would have followed the agent — so it is opt-in.

A redirect from upstream is handed back, never followed: Go would re-send the
credential to wherever it points. One that points back at the upstream is made
relative first, so the address stays the edge's (rule 18) — as does the one an
unreachable upstream would otherwise quote in its error.

Serves `/anthropic`, `/openai`, `/genai` (namespaced per
[`gateways/RULES.md`](../../../.agents/gateways/RULES.md) rule 5) plus
`/health`; any other path forwards on the `openai` wire. `edge health` exits
0 when `/health` answers on `LISTEN`, for a shell-free readiness probe.
