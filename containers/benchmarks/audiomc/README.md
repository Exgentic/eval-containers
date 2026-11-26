# audiomc

Audio MultiChallenge (Scale AI): multi-turn spoken dialogue. The model hears
every user turn as audio and answers the final turn. A judge grades the answer
against per-conversation rubrics. It is model-only and needs a model that takes
audio input.

**Status:** Not yet released. No replay fixture has landed (see
[`.agents/benchmarks/RULES.md`](../../../.agents/benchmarks/RULES.md) rule 21a).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 452 conversations (3–8 user turns, 47 speakers), 1,712 rubrics |
| Axes | Inference Memory 132 (41 of them Audio-Cue) · Instruction Retention 120 · Voice Editing 117 · Self Coherence 83 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Evaluation | **model-only**, through a native harness (no agent) |
| Upstream | [huggingface.co/datasets/ScaleAI/audiomc](https://huggingface.co/datasets/ScaleAI/audiomc) · [arXiv 2512.14865](https://arxiv.org/abs/2512.14865) |
| License | MIT (ungated) |
| Dataset revision | `90ea11040fd05f41cf433b90f97ada45d847c500` |
| Agent | `native`. The harness is the benchmark's own and is the only pairing (rule 12b) |
| Model | must accept `input_audio` on the OpenAI chat surface (e.g. Gemini) |

## What this is

Audio MultiChallenge extends the text MultiChallenge axes (Inference Memory,
Instruction Retention, Self Coherence) to speech. It adds **Voice Editing**,
which tests mid-utterance repairs and backtracking. Its **Audio-Cue**
Inference Memory items ask about ambient sounds and paralinguistic signals that
a transcript does not carry.

The protocol is fixed-context: the whole conversation up to the final user turn
is pre-seeded, and only the model's reply to that last turn is graded. In the
paper's words, "user turns are provided in context as base64-encoded audio
files, while assistant turns are provided as raw text". No system prompt is
sent.

## Native harness

A coding agent cannot listen to a wav, so the image ships `/harness.sh`
(`run_audiomc.py`, `LABEL eval.benchmark.agent="native"`). The launcher runs it
as root with `EVAL_TASK_ID`. It reads `/tasks/<id>/turns.json` and sends one
request to the gateway's OpenAI-compatible surface:

```
POST $OPENAI_BASE_URL/chat/completions
{ "model": $MODEL,
  "messages": [
    {"role":"user","content":[{"type":"input_audio","input_audio":{"data":<base64 user_1.wav>,"format":"wav"}}]},
    {"role":"assistant","content":"<dataset assistant turn 1>"},
    …
    {"role":"user","content":[{"type":"input_audio","input_audio":{"data":<base64 user_N.wav>,"format":"wav"}}]}
  ] }
```

The harness prints the reply, and the launcher keeps it as
`/output/agent/stdout.log`. When no reply comes back it prints nothing, so the
task scores 0. Upstream's history builder skips an empty assistant turn, and
the harness does the same. One conversation has one.

The audio is 48 kHz wav, kept byte-for-byte as upstream ships it. The largest
conversation is about 92 MB once base64'd. The image therefore raises the
edge's request cap to 128 MiB (`EDGE_MAX_REQUEST_BYTES`; the default of 64 MiB
would refuse 9 conversations). That still fits Gemini's 100 MB inline-request
limit and bifrost's default 100 MB body limit.

Knobs (runtime env, read by the harness):

| Var | Default | Why |
|-----|---------|-----|
| `AUDIOMC_TIMEOUT` | `EVAL_TIMEOUT` − 60 (240) | Per-request seconds, so the request gives up before the harness is killed. |
| `AUDIOMC_EXTRA_BODY` | unset | JSON object merged into the request, e.g. a thinking config. The paper reports "(Thinking)" variants of several models, and no spelling of that is portable. |

Set them per run with `docker run -e` (single), a compose override file that
adds them to the runner's `environment:` (compose's runner does not pass the
shell environment through), or the chart's `runnerExtraEnv` (k8s).

## How it's graded

`/grade.sh` runs `grade.py`, which is upstream's judge. It uses the judge prompt
from the dataset card verbatim, sent as the only message. It also uses
upstream's `build_grading_conversation_history`: user turns as their
transcripts, earlier assistant turns from the dataset, and the model's reply as
the final assistant turn. Each rubric item is judged in its own call.

- **reward = 1.0 iff every rubric item is met**, else 0.0. This is the paper's
  primary metric, APR (Average Pass Rate), once averaged over tasks.
- The per-rubric fraction, ARS (Average Rubric Score), is recorded with each
  verdict and explanation in `/logs/verifier/grading_details.json`.

**Judge model, a deviation from upstream.** Upstream judges with `o4-mini`.
Like every other rubric-judged benchmark here, this judge is the run's pinned
model, called as `"model": "default"` through the same gateway. A Gemini run is
therefore judged by Gemini, and scores are not directly comparable with the
paper's.

## Files

- `Dockerfile` builds the benchmark image. It fetches the pinned parquet and
  writes one flat directory per task: `turns.json`, `user_<n>.wav`,
  `rubric.json`, `id.txt` and `axis.txt`, all root-only.
- `run_audiomc.py` is the native harness: it sends the conversation and prints
  the reply.
- `grade.py` is the rubric judge, root-only at `/tests/grade.py`.
- `compose.yaml` is the compose-mode deployment. It extends the shared
  `compose/runner-native.yaml` and adds no services.
- single: the standalone bundle, from the generic `core/standalone.Dockerfile`.
- k8s: the shared chart `benchmarks/_chart`, `--set benchmark=audiomc`.
  `presets/audiomc.yaml` pins `agent: native`.
- `README.md` is this file.
- `AUDIT.md` is the standing audit record.

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running: three deployment surfaces

| Mode | File | Invocation |
|------|------|------------|
| **single** | `core/standalone.Dockerfile` | `docker run -e OPENAI_API_KEY=… -e OPENAI_API_BASE=… -e EVAL_MODEL=… <image>-standalone` |
| **compose** | `compose.yaml` | `docker compose -f benchmarks/audiomc/compose.yaml up` |
| **k8s** | shared chart | `helm template audiomc benchmarks/_chart --set benchmark=audiomc \| kubectl apply -f -` |

```bash
# Compose mode, against an audio-capable model
EVAL_MODEL=gcp/gemini-3-flash-preview OPENAI_API_KEY=… OPENAI_API_BASE=… \
  docker compose -f benchmarks/audiomc/compose.yaml up

# A different conversation (rule 24c: parameterized via ${TASK_ID:-0})
TASK_ID=42 docker compose -f benchmarks/audiomc/compose.yaml up

# k8s, a different conversation
helm template audiomc benchmarks/_chart \
  --set benchmark=audiomc --set task=42 | kubectl apply -f -
```

## Build args

To rebuild the eval image from source instead of pulling it:

```bash
docker build -f core/combination.Dockerfile \
  --build-arg BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/audiomc:latest \
  --build-arg AGENT_IMAGE=ghcr.io/exgentic/agents/native:latest \
  --build-arg MODEL_IMAGE=ghcr.io/exgentic/models/bifrost:latest \
  -t ghcr.io/exgentic/evals/audiomc--native:latest .
```

`BENCHMARK_VERSION` (default above) selects the dataset revision at build time
(rule 4).
