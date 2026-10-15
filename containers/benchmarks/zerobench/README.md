# zerobench

ZeroBench - impossible visual reasoning benchmark for large multimodal models
(ICML 2026). **Model-only, native harness** — faithful to the upstream protocol
(no agent).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 100 (main set) |
| Environment | shared-env |
| Agent | native (model-only, benchmarks/RULES.md 12a) |
| Modality | multimodal (vision + text) |
| Internet required | false |
| Released | no |
| Upstream | [HF dataset](https://huggingface.co/datasets/jonathan-roberts1/zerobench) · [code (MIT)](https://github.com/jonathan-roberts1/zerobench) |
| Paper | [arXiv:2502.09696](https://arxiv.org/abs/2502.09696) |
| Leaderboard | [zerobench.github.io](https://zerobench.github.io/#leaderboards) |
| Dataset revision | `8fcdfd9138c802cac52b774b36946e70f738f500` |

## What it does (native harness)

ZeroBench is single-turn vision-language inference, not an agent task, so this
benchmark runs as a **native harness** (`/harness.sh` → `run_zerobench.py`) that
reproduces the upstream protocol directly against the model — there is no agent.
For one question (`EVAL_TASK_ID`) it:

1. Builds the multimodal prompt: the question text + the upstream instruction
   *"Let's think step by step and give the final answer in curly braces, like
   this: {final answer}"* + the question's 1–3 image(s) as `image_url` parts.
2. Draws **N = 5 samples** from the model via the gateway's OpenAI-compatible
   `/chat/completions`. Sampling knobs (env): `ZEROBENCH_N`, `ZEROBENCH_TEMP`,
   `ZEROBENCH_TOP_P` (set a knob to the empty string to omit it — reasoning
   models force temperature 1), `ZEROBENCH_MAX_TOKENS`, and
   `ZEROBENCH_EXTRA_BODY` (a JSON object merged into the request for
   provider-specific knobs). Defaults are the release-era settings
   (temp 0.7, top_p 0.95, no thinking). **To match the current leaderboard**
   (thinking models, default params) enable the model's extended thinking, e.g.
   for Bedrock Claude Opus 4.8: `ZEROBENCH_TEMP=1 ZEROBENCH_TOP_P=''
   ZEROBENCH_MAX_TOKENS=60000
   ZEROBENCH_EXTRA_BODY='{"thinking":{"type":"adaptive"},"output_config":{"effort":"high"}}'`.
3. Extracts the last `{...}` from each sample (upstream regex `\{(.*?)\}`) and
   scores it case-insensitively against the gold answer.

## Metrics

Per question it records the three leaderboard metrics and exports them to
`/output/task/zerobench.json`:

- **pass@1** — mean fraction of the 5 samples correct (also written as the
  reward, so the swept mean reward over all 100 questions == fleet pass@1)
- **pass@5** — 1 if ≥1 of 5 samples correct
- **pass^5** — 1 if all 5 samples correct

A sweep over all 100 tasks aggregates these into the fleet-level
pass@1 / pass@5 / pass^5 that the leaderboard reports.

## Gated data

`jonathan-roberts1/zerobench` is license-gated on HuggingFace. The build fetches
it with an ephemeral `--mount=type=secret,id=HF_TOKEN` (never baked —
benchmarks/RULES.md 8a), the same mechanism `gaia`/`hle`/`flores200` use. A build
host (and CI) needs an `HF_TOKEN` whose account accepted the dataset's gate. See
[`tests/build/known-broken.md`](../../../tests/build/known-broken.md).

## Files

- `Dockerfile` — builds the benchmark image (gated fetch → native harness)
- `run_zerobench.py` — the native harness (model-only, 5-sample, pass@1/5/^5)
- `compose.yaml` — compose file (native runner) for `eval-containers run zerobench`
- `docker-bake.hcl` — bake target `benchmark-zerobench` (HF_TOKEN secret)
- `README.md` / `AUDIT.md`
