# mmmu-pro

MMMU-Pro, the harder MMMU: questions filtered so text-only models cannot answer
them, with the options augmented to up to 10. It runs **model-only** through a
native harness that reproduces upstream's protocol, because published MMMU-Pro
scores are single-call model inference with no agent. This image holds the
**standard** setting: 10 options by default, the one model cards report as
"MMMU Pro (Standard 10)", and 4 options as a knob. The screenshot setting is
[`mmmu-pro-vision`](../mmmu-pro-vision/). Both run the same harness,
[`core/mmmu-pro-harness`](../../core/mmmu-pro-harness/run_mmmu_pro.py).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1730 (test split) |
| Environment | shared-env |
| Agent | native (model-only, benchmarks/RULES.md 12a) |
| Modality | multimodal (text + 1–7 images per question) |
| Internet required | false |
| Released | no |
| Upstream | [MMMU-Benchmark/MMMU `mmmu-pro/`](https://github.com/MMMU-Benchmark/MMMU/tree/b26bdefc7e5e007962fde8f93b2bb53c99c8d13d/mmmu-pro) @ `b26bdefc` · [HF `MMMU/MMMU_Pro`](https://huggingface.co/datasets/MMMU/MMMU_Pro) (Apache-2.0) |
| Paper | [arXiv:2409.02813](https://arxiv.org/abs/2409.02813) |
| Leaderboard | [mmmu-benchmark.github.io](https://mmmu-benchmark.github.io/#leaderboard) |
| Dataset revision | `0d7426df4ccb3d8704a992fe6850abee7c262127` |

## Configurations

| Setting | Benchmark | `MMMU_PRO_SETTING` | What the model gets |
|---------|-----------|--------------------|---------------------|
| standard (10 options) | `mmmu-pro` | `standard (10 options)` (default) | question + lettered options + prompt, then its images |
| standard (4 options) | `mmmu-pro` | `standard (4 options)` | the same questions and images with upstream's 4 options |
| vision | `mmmu-pro-vision` | `vision` (its only one) | the vision prompt, then one screenshot of the question and options |

`MMMU_PRO_PROMPT` selects `cot` (default) or `direct`. The paper runs both and
reports the higher one. Its leaderboard "Overall" is the mean of standard (10
options) and vision. Standard (4 options) is reported "solely for comparison".
Task `i` is the same question in all three settings.

## Protocol (upstream file:line at `b26bdefc`)

- **Prompt:** `infer/infer_gpt.py:43-56` (`construct_prompt`) and
  `prompts.yaml`, verbatim. That is the question, then `A. …` / `B. …` one per
  line, then the standard prompt.
- **Images:** `infer_gpt.py:38-41, 58-62`. Each `<image N>` becomes `<image>`,
  and `image_N` is sent once per occurrence, in token order, as `image_url`
  parts after the text (`infer_gpt.py:95-138`). An image the text never
  references is not sent.
- **Vision:** `infer_gpt.py:162-170`. The prompt is `prompts.yaml`'s `vision`
  entry alone, with the single screenshot.
- **Request:** one user message, `max_tokens` 4096, no `temperature`/`top_p`
  (`infer_gpt.py:116-138, 170`). One sample per question.
- **Scoring:** `evaluate.py:16-33, 215-290, 429-443`
  (`parse_multi_choice_response`), checked against upstream on synthetic
  responses. The reward is 1.0 if the parsed letter equals the gold one, else
  0.0, so the mean reward is upstream's accuracy. Per-question detail goes to
  `/output/task/mmmu_pro.json`: setting, prompt, knobs, gold, parsed, `guessed`,
  error, and the response tail.

## Where this differs from upstream, and why

- **The random fallback is seeded per question.** When the parser finds no
  answer, upstream picks `random.choice` from a generator seeded once with 42
  (`evaluate.py:13`). So a question's guess depends on how many guesses came
  before it in the file, which a one-question-per-run harness cannot
  reproduce. Here the guess comes from `random.Random("42:<upstream id>")`:
  the same guess every run, and the same expected value as upstream's.
  Scoring it as wrong instead would bias scores below published ones. The
  export marks each guess (`guessed: true`), so a guess-free score can be
  recomputed.
- **A failed request scores 0.** Upstream replaces an API error with an empty
  response and guesses (`evaluate.py:18-19`). A gateway or network failure
  measures nothing about the model, so here it scores 0 and is recorded under
  `error`. An empty reply still goes to the parser, and so to the guess, as
  upstream does.
- **No system message.** `infer_gpt.py:127` sends a "You are ChatGPT …" system
  prompt, a GPT-specific identity. Upstream's other harnesses
  (`infer_gemini.py`, `infer_lmdeploy.py`) send none, and neither does this one.
- **Correct image type.** Images are sent as `image/png`, which is what they
  are. Upstream labels its PNG re-encode `image/jpeg` (`infer_gpt.py:109`).
- **Standard (4 options) is supported.** Upstream's `process_prompt` only
  branches on `standard (10 options)` and `vision`, and the 4-option setting
  falls through. The paper reports 4 options anyway, and `construct_prompt` does
  not depend on the setting, so 4 options takes the standard path.

## Knobs (env)

| Variable | Default | Meaning |
|----------|---------|---------|
| `MMMU_PRO_SETTING` | per image (see above) | which setting to run |
| `MMMU_PRO_PROMPT` | `cot` | `cot` or `direct` |
| `MMMU_PRO_TEMP` | empty (omitted) | `temperature`; empty omits it, like upstream |
| `MMMU_PRO_TOP_P` | empty (omitted) | `top_p`; empty omits it |
| `MMMU_PRO_MAX_TOKENS` | `4096` | `max_tokens`, upstream's value |
| `MMMU_PRO_TIMEOUT` | `300` | HTTP timeout for the call, in seconds |
| `MMMU_PRO_EXTRA_BODY` | empty | a JSON object merged into the request, for provider knobs |

The harness runs privileged with the container environment whole (rule 12c),
so on k8s the knobs go in through the chart's `runnerExtraEnv`. Bound the run
with `EVAL_TIMEOUT` above `MMMU_PRO_TIMEOUT`. The preset sets 600 s.

**Matching a vendor's model card.** A card's number is usually the model's
recommended sampling with thinking on, not upstream's 4096-token cap. For a
reasoning model, for example:

```bash
MMMU_PRO_PROMPT=cot MMMU_PRO_TEMP=1.0 MMMU_PRO_MAX_TOKENS=65536 \
MMMU_PRO_TIMEOUT=1800 MMMU_PRO_EXTRA_BODY='{"reasoning_effort":"xhigh"}'
```

Raise `EVAL_TIMEOUT` to match. Use the provider's own field in `EXTRA_BODY`
(e.g. `{"thinking":{"type":"adaptive"}}` for Claude).

## Run

```bash
eval-containers build bench mmmu-pro
eval-containers build eval mmmu-pro --agent native
eval-containers run mmmu-pro --agent native --model <model> --task-id 0 --local
```

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Files

- `Dockerfile`: fetches the pinned standard (10 and 4 options) shards. Rows go to `/tasks/all.jsonl` and images to `/tasks/images/<task>_<n>.png`, both root-only.
- `compose.yaml`: native runner for `eval-containers run mmmu-pro`
- `docker-bake.hcl`: bake target `benchmark-mmmu-pro`
- `README.md` / `AUDIT.md`
