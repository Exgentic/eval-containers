# mmmu-pro-vision

MMMU-Pro's **vision** setting. It has the same 1730 questions as
[`mmmu-pro`](../mmmu-pro/), but each is a screenshot or photo with the question
and its options embedded. The model gets only that image and upstream's vision
prompt (`prompts.yaml`, via `infer/infer_gpt.py:162-170`). It runs model-only
through the same native harness
([`core/mmmu-pro-harness`](../../core/mmmu-pro-harness/run_mmmu_pro.py)) and
the same answer parser.

It is a separate image only because the screenshots alone are 1.6 GB. Protocol,
deviations from upstream, knobs (`EVAL_MMMU_PRO_PROMPT`, `EVAL_MMMU_PRO_TEMP`, …) and how
to match a model card are all in [`mmmu-pro`'s README](../mmmu-pro/README.md).
The leaderboard's MMMU-Pro "Overall" is the mean of this benchmark and
`mmmu-pro`'s standard (10 options) setting.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1730 (test split) |
| Environment | shared-env |
| Agent | native (model-only, benchmarks/RULES.md 12a) |
| Modality | multimodal (one screenshot per question) |
| Internet required | false |
| Released | no |
| Upstream | [MMMU-Benchmark/MMMU `mmmu-pro/`](https://github.com/MMMU-Benchmark/MMMU/tree/b26bdefc7e5e007962fde8f93b2bb53c99c8d13d/mmmu-pro) · [HF `MMMU/MMMU_Pro`](https://huggingface.co/datasets/MMMU/MMMU_Pro) config `vision` (Apache-2.0) |
| Paper | [arXiv:2409.02813](https://arxiv.org/abs/2409.02813) |
| Dataset revision | `0d7426df4ccb3d8704a992fe6850abee7c262127` |

## Run

```bash
eval-containers build bench mmmu-pro-vision
eval-containers build eval mmmu-pro-vision --agent native
eval-containers run mmmu-pro-vision --agent native --model <model> --task-id 0 --local
```

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Files

- `Dockerfile`: fetches the pinned vision shards. Rows go to `/tasks/all.jsonl` and screenshots to `/tasks/images/<task>.png`, both root-only.
- `compose.yaml`: native runner for `eval-containers run mmmu-pro-vision`
- `docker-bake.hcl`: bake target `benchmark-mmmu-pro-vision`
- `README.md` / `AUDIT.md`
