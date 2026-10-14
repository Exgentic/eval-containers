# zerobench-agent

ZeroBench (agent variant) — the **with-agent** configuration of ZeroBench (ICML
2026). Same 100-question main set as the model-only [`zerobench`](../zerobench/),
but solved by a tool-using **agent** (e.g. Claude Code) instead of the native
model-only harness. Maps to the leaderboard's externally-reported
"Claude Code harness tools" (tool-assisted) entries.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 100 (main set) |
| Environment | shared-env |
| Agent | any (agent-facing); e.g. `claude-code` |
| Modality | multimodal (vision + text) |
| Internet required | false |
| Released | no |
| Upstream | [HF dataset](https://huggingface.co/datasets/jonathan-roberts1/zerobench) · [code (MIT)](https://github.com/jonathan-roberts1/zerobench) |
| Dataset revision | `8fcdfd9138c802cac52b774b36946e70f738f500` |

## Model-only vs agent — two benchmarks, one scorer

- **`zerobench`** (native harness) — model-only, "no agent" (the `native` agent).
- **`zerobench-agent`** (this) — agent-facing; the agent sees the question + its
  1–3 image files in `/app` and must end with the answer in `{braces}`.

Both score through the **same shared grader** `core/test-braced-match`
(`score_braced.py`): extract the last `{...}`, case-insensitive match to gold.
The native harness imports its `extract_answer`/`is_correct`; this benchmark runs
its `/test.sh` as `/grade.sh`. One implementation, no drift.

## Gated data

`jonathan-roberts1/zerobench` is license-gated on HuggingFace; the build fetches
it via an ephemeral `--mount=type=secret,id=HF_TOKEN` (benchmarks/RULES.md 8a).
See [`tests/build/known-broken.md`](../../../tests/build/known-broken.md).

## Files

- `Dockerfile` — gated fetch → agent-facing task (images in `/app`) + shared grader
- `compose.yaml` — compose runner for `eval-containers run zerobench-agent`
- `docker-bake.hcl` — bake target `benchmark-zerobench-agent`
- `README.md` / `AUDIT.md`
