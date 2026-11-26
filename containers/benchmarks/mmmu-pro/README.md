# mmmu-pro

MMMU-Pro, the harder MMMU: questions filtered so text-only models cannot answer
them, with the options augmented to up to 10. This image is the **standard
(10 options)** setting — the one model cards report as "MMMU Pro (Standard 10)".

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1730 (test split) |
| Environment | shared-env |
| Modality | multimodal (text + 1–7 images per question) |
| Internet required | false |
| Released | no |
| Upstream | [MMMU-Benchmark/MMMU `mmmu-pro/`](https://github.com/MMMU-Benchmark/MMMU/tree/main/mmmu-pro) · [HF dataset `MMMU/MMMU_Pro`](https://huggingface.co/datasets/MMMU/MMMU_Pro) (Apache-2.0) |
| Paper | [arXiv:2409.02813](https://arxiv.org/abs/2409.02813) |
| Dataset revision | `0d7426df4ccb3d8704a992fe6850abee7c262127` (config `standard (10 options)`) |

## What the agent sees

Upstream's standard-setting prompt (`construct_prompt` in `mmmu-pro/infer/`): the
question, the options lettered `A.`, `B.`, … one per line, then the direct prompt
from `mmmu-pro/prompts.yaml`, `Answer with the option letter from the given
choices directly.` Two lines follow it: `Print only the answer letter, nothing
else.`, because the grader is exact match, and a list mapping each `<image N>`
in the text to its file, `/app/image_N.png`. Only the images the text
references are copied, which matches what upstream sends.

## How it's graded

The shared `core/test-exact-match` scorer compares the agent's stdout with the
gold letter (`answer.txt`), so 1 or 0 per task. The score is the mean over the
1730 tasks, as upstream's `evaluate.py` computes it.

## Comparing with published numbers

- **Direct, not CoT.** Upstream has a direct prompt and a CoT prompt. The paper
  runs both and reports the higher one. The CoT prompt ends in `Answer: $LETTER`,
  which needs upstream's response parser, so this image uses the direct prompt.
  A reasoning model or an agent still reasons before it answers.
- **Stricter parsing.** Upstream's `parse_multi_choice_response` accepts
  `(B)`, `B.` or the option text, and when it finds no answer it picks a random
  letter. Exact match accepts only the letter, so a badly formatted answer here
  scores 0 instead of getting a 1-in-10 guess.
- **Standard (10) only.** The leaderboard's "Overall" score is the average of
  this setting and **vision** (the question as a screenshot). Vision is not
  included yet. Its upstream prompts ask for an `Answer: $LETTER` last line,
  which `test-exact-match` cannot grade.
- Not every question has 10 options: 1213 have 10, and the rest have between
  2 and 12. The letters run as far as the options go.

## Run

```bash
eval-containers build bench mmmu-pro
eval-containers oracle mmmu-pro --task-id 0 --local
eval-containers run mmmu-pro --agent claude-code --model <model> --task-id 0 --local
```

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Files

- `Dockerfile` — fetches the pinned parquet shards, builds `/tasks/all.jsonl`, and holds the entrypoint and grader
- `compose.yaml` — compose runner for `eval-containers run mmmu-pro`
- `docker-bake.hcl` — bake target `benchmark-mmmu-pro`
- `README.md` / `AUDIT.md`
