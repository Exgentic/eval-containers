# longbench-v2

LongBench v2 is a long-context benchmark: 503 four-choice questions (A/B/C/D)
over real-world contexts of 8k to 2M words, in six categories:

- single-document QA
- multi-document QA
- long in-context learning
- long-dialogue history
- code repositories
- long structured data

It runs **model-only, as a native harness**. Published scores come from
upstream's `pred.py`, which makes one chat call per question with no agent, so
this benchmark reproduces that call.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 503 (ids `0`–`502` in dataset order; upstream `_id` in `id.txt`) |
| Environment | shared-env |
| Agent | native (model-only, benchmarks/RULES.md 12a) |
| Internet required | false |
| Released | no |
| Upstream | [THUDM/LongBench](https://github.com/THUDM/LongBench) (MIT; LongBench v2 at the repo root) |
| Dataset | [zai-org/LongBench-v2](https://huggingface.co/datasets/zai-org/LongBench-v2) (Apache-2.0; formerly `THUDM/LongBench-v2`) |
| Paper | [arXiv:2412.15204](https://arxiv.org/abs/2412.15204) |
| Leaderboard | [longbench2.github.io](https://longbench2.github.io/#leaderboard) |
| Dataset revision | `2b48e494f2c7a2f0af81aae178e05c7e1dde0fe9` |
| Protocol source | THUDM/LongBench `2e00731f8d0bff23dc4325161044d0ed8af94c1e`: `pred.py`, `prompts/0shot.txt`, `config/model2maxlen.json`, `result.py` |

## Protocol (upstream `pred.py`, 0-shot)

For one question (`EVAL_TASK_ID`), `/harness.sh` (`run_longbench_v2.py`) does
the following:

1. **Prompt.** It uses `prompts/0shot.txt` verbatim, filled with the stripped
   `context`, `question` and `choice_A`–`choice_D` in `pred.py`'s
   substitution order (`get_pred`). The prompt is rendered at build time by
   `materialize.py` and is byte-identical to `pred.py`'s.
2. **Truncation.** This follows `query_llm`. The whole prompt is tokenized, and
   if it is longer than `max_len`, the first and last `max_len//2` tokens are
   kept. `max_len` is 120000, which is `config/model2maxlen.json`'s value for
   every model except Claude 3.5 Sonnet (200000).
   - **Tokenizer:** upstream uses the served model's own Hugging Face tokenizer
     for open models. For API models it uses tiktoken's `gpt-4o-2024-08-06`
     encoding (`o200k_base`, with `disallowed_special=()`). This image cannot
     carry every served model's tokenizer, so **every model uses the API-model
     path**. For an open model, the cut therefore falls at roughly, not exactly,
     the same place. A model whose tokenizer is less efficient than
     `o200k_base` can then overflow a 128k served window, and its request fails
     where upstream's would have fit. Lower `EVAL_LONGBENCH_V2_MAX_LEN` for such
     a model.
3. **Call.** One `chat/completions` request through the gateway, with the
   prompt as the only user message, `temperature=0.1` and `max_tokens=128`.
   It makes up to 5 tries, 1 s apart.
4. **Answer extraction.** Upstream's `extract_answer`, verbatim: drop every
   `*`, take the *first* `The correct answer is \(([A-D])\)`, else the first
   `The correct answer is ([A-D])`.
5. **Score.** The reward is `1` iff the extracted letter equals `answer`, else
   `0`. The mean over the 503 tasks is the leaderboard's "Overall" accuracy
   (random guessing scores 25%).

Upstream never judges a question whose every try failed or whose reply was
empty (`if output == '': continue`), so it falls out of the denominator. Here an
empty reply scores `0` with `error: "empty reply"`. A question whose every try
failed exits non-zero, so the attempt is recorded as errored and a resumed run
retries it.

`/output/task/longbench-v2.json` records the following for each task:

- `pred`, `judge`, the full reply, and `error`
- `prompt_tokens` and `truncated`
- `domain`, `sub_domain`, `difficulty` and `length`, so a sweep can compute
  `result.py`'s Easy/Hard and Short/Medium/Long splits

### Not reproduced

- `pred.py --cot` is not implemented. It is the two-call scaffold
  (`0shot_cot.txt`, then `0shot_cot_ans.txt`) behind the leaderboard's "w/ CoT"
  column for non-reasoning models.
- The leaderboard lists reasoning models (o1, DeepSeek-R1, …) only under
  "w/ CoT", because they "natively use CoT". Their request settings are not in
  `pred.py`, and a 128-token cap cannot hold their reasoning. The defaults here
  reproduce the **0-shot ("w/o CoT") setting**. For a reasoning model, keep the
  same single 0-shot call and use its vendor's settings (below).

## Sampling knobs

All knobs are env vars on the runner, in the `EVAL_` namespace, so the run's
recorded configuration includes them. Unset or empty means upstream's default.
`none` omits the parameter from the request, so the model's own default
applies; this works for every knob except `MAX_LEN`.

| Var | Default (upstream) | Meaning |
|-----|--------------------|---------|
| `EVAL_LONGBENCH_V2_MAX_LEN` | `120000` | truncation window, in tokens (`model2maxlen.json`) |
| `EVAL_LONGBENCH_V2_TEMP` | `0.1` | `temperature` |
| `EVAL_LONGBENCH_V2_TOP_P` | `none` | `top_p` (upstream doesn't send it) |
| `EVAL_LONGBENCH_V2_MAX_TOKENS` | `128` | `max_tokens` (the gateway forwards it as `max_completion_tokens` where the provider needs it) |
| `EVAL_LONGBENCH_V2_TIMEOUT` | `600` | per-try HTTP timeout, in seconds (the OpenAI client's default) |
| `EVAL_LONGBENCH_V2_EXTRA_BODY` | `{}` | JSON object merged into the request, e.g. `{"reasoning_effort":"high"}` |

To match a vendor's model card for a reasoning model, use the vendor's own
sampling settings and a cap large enough for its reasoning. For example:

```bash
EVAL_LONGBENCH_V2_TEMP=1.0
EVAL_LONGBENCH_V2_MAX_TOKENS=65536
EVAL_LONGBENCH_V2_EXTRA_BODY='{"reasoning_effort":"xhigh"}'
```

If the model is served with a larger window than 128k, set `MAX_LEN` to that
window minus `MAX_TOKENS`.

How to set the knobs on each surface:

- **Compose:** export them in the shell. `compose.yaml` forwards all six.
- **k8s:** `--set runnerExtraEnv[0].name=EVAL_LONGBENCH_V2_TEMP --set runnerExtraEnv[0].value=1.0`
- **Single container:** `docker run -e …`

The run is bounded at 3600 s in both `compose.yaml` and the chart preset. That
covers five tries of 600 s. For long reasoning calls, raise `EVAL_TIMEOUT`
(and `EVAL_LONGBENCH_V2_TIMEOUT`).

## Oracle

`solution.sh` grades a reply in the required format that carries the gold
letter, using the harness's own `--score` path. It makes no model call. A no-op
leaves `/grade.sh`'s fail-closed `0`.

```bash
eval-containers oracle longbench-v2 --local   # gold=1, no-op=0
```

## Eval image build args

`core/combination.Dockerfile` takes these build args:

- `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/longbench-v2:latest`
- `AGENT_IMAGE=ghcr.io/exgentic/agents/native:latest`
- `AGENT_VERSION=<native version>`

## Files

- `Dockerfile` — builds the benchmark image: the dataset, tiktoken with its cached encoding, and the harness
- `materialize.py` — build-time loader that renders `0shot.txt` per task into `/tasks/all.jsonl`
- `run_longbench_v2.py` — the native harness: truncation, the model call, extraction and the judge
- `solution.sh` — oracle gold, mounted at oracle time and never baked into the image
- `compose.yaml` — compose file (native runner) for `eval-containers run longbench-v2`
- `docker-bake.hcl` — bake target `benchmark-longbench-v2`
