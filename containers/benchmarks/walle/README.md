# walle

walle (MoonshotAI) — JSON-Schema `response_format` conformance probe: does the
served stack accept valid schemas and reject invalid ones? Stack-correctness,
model-only.

**Status:** Not yet released — no replay fixture landed (see
[`.agents/benchmarks/RULES.md`](../../../.agents/benchmarks/RULES.md) rule 21a).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 395 (212 valid + 183 invalid, across 16 feature suites) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Evaluation | **model-only** conformance probe (no agent) |
| Upstream | [github.com/MoonshotAI/walle](https://github.com/MoonshotAI/walle) |
| License | MIT |
| Dataset revision | `cc1c6b7dab5496d5184677ecf4c3b95fc1bd1606` (tag v0.1.10) |
| Canonical agent | `claude-code` (naming only; there is no agent) |

## What this is

walle is a corpus of JSON Schemas used inside
[MoonshotAI/Kimi-Vendor-Verifier](https://github.com/MoonshotAI/Kimi-Vendor-Verifier)
to check that a model-serving deployment handles structured output
(`response_format`) faithfully. It is **not** an agent-capability benchmark —
it probes **stack correctness**: given a JSON Schema, does the endpoint's
constrained-decoding backend do the right thing?

- **valid** schemas (212) exercise the JSON-Schema surface a correct stack must
  support: `$ref`, `$defs`, `anyOf`, nested defs, range constraints,
  `additionalProperties`, enums, and so on. A correct stack **accepts** them.
- **invalid** schemas (183) are malformed. A correct stack **rejects** them
  (HTTP 400/422) rather than silently accepting.

The 16 suites: `TestAdditionalProperties`, `TestAnyOf`, `TestBasicTypes`,
`TestDefs`, `TestDescription`, `TestEnforcerCases`, `TestID`,
`TestKeywordsValidation`, `TestNestedDefsDepth`, `TestNumberFormat`,
`TestRangeConstraints`, `TestRefInProperties`, `TestReferences`, `TestRequired`,
`TestSingleTypeInArray`, `TestTypeLocation`.

## Model-only conformance probe

There is no repo agent and no user simulator. The runner container (which holds
`EVAL_TASK_ID`, withheld from the scrubbed agent phase per rule 7) resolves the
sequential case id to its schema via the build-time map (`/tasks/all.jsonl`),
then POSTs it to the gateway's OpenAI-compatible surface:

```
POST $OPENAI_BASE_URL/chat/completions
{ "model": $MODEL,
  "messages": [{"role":"user","content":"Return a JSON object that conforms to the provided schema."}],
  "response_format": {"type":"json_schema","json_schema":{"name":"walle_case","schema": <the case schema>}} }
```

It classifies the outcome — **accept** (2xx), **reject** (HTTP 400/422), or
**error** (auth / wrong endpoint / 5xx / timeout / connection) — and records it
plus a per-case diagnostic in `/output/task/walle.json`.

`WALLE_STRICT=true` adds `"strict": true` to the `json_schema` (OpenAI strict
mode — narrows to a smaller schema subset; off by default so the probe reflects
the stack's general `response_format` support, matching walle's own contract).

## How it's graded

**reward = 1.0 iff the observed verdict matches the expected verdict, else
0.0.** Valid schemas must be accepted; invalid schemas must be rejected. An
`error` (infra failure, not a verdict on the schema) scores 0 and is flagged as
such — a broken gateway reads as an error, not a conformance fail.

For a valid + accepted case the probe additionally validates the returned
content against the schema with `jsonschema` and records `conforms` as a
**diagnostic** in `walle.json`. It does **not** affect the reward: whether the
model's output conforms is a model / constrained-decoding-quality question,
while walle's contract is schema *acceptance*.

`run_walle.py` writes the reward to `/logs/verifier/reward.txt`; the shared
`write-result` derives `passed` as `reward == 1.0` and emits `result.json`.
Sweeping every task gives the stack's accept/reject conformance rate — a clean
way to compare structured-output support across serving stacks (vLLM, SGLang,
llm-d, a vendor endpoint) on the same corpus.

## Files

- `Dockerfile` — builds the benchmark image (`FROM python:3.12-slim`; fetches
  the pinned corpus; assembles the case map; installs `jsonschema`).
- `run_walle.py` — single-case probe (resolve id → schema, POST as
  `response_format`, classify, write reward + diagnostic).
- `compose.yaml` — compose-mode deployment (runner entrypoint override; no
  bespoke services).
- single — the standalone bundle, from the generic `core/standalone.Dockerfile`.
- k8s — the shared chart `benchmarks/_chart`, `--set benchmark=walle`
  (`presets/walle.yaml` for the probe entrypoint).
- `README.md` — this file.
- `AUDIT.md` — standing audit record.

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running — three deployment surfaces

| Mode | File | Invocation |
|------|------|------------|
| **single** | `core/standalone.Dockerfile` | `docker run -e OPENAI_API_KEY=… -e OPENAI_API_BASE=… <image>-standalone` |
| **compose** | `compose.yaml` | `docker compose -f benchmarks/walle/compose.yaml up` |
| **k8s** | shared chart | `helm template walle benchmarks/_chart --set benchmark=walle \| kubectl apply -f -` |

```bash
# Compose mode
OPENAI_API_KEY=… OPENAI_API_BASE=… \
  docker compose -f benchmarks/walle/compose.yaml up

# A different case (rule 24c — parameterized via ${TASK_ID:-0})
TASK_ID=42 docker compose -f benchmarks/walle/compose.yaml up

# k8s, a different case
helm template walle benchmarks/_chart \
  --set benchmark=walle --set task=42 | kubectl apply -f -
```

## Build args

To rebuild the eval image from source (instead of pulling):

```bash
docker build -f core/combination.Dockerfile \
  --build-arg BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/walle:latest \
  --build-arg AGENT_IMAGE=ghcr.io/exgentic/agents/claude-code:latest \
  --build-arg AGENT_VERSION=2.1.0 \
  --build-arg MODEL_IMAGE=ghcr.io/exgentic/models/bifrost:latest \
  -t ghcr.io/exgentic/evals/walle--claude-code:latest .
```
