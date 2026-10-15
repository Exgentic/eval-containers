# zerobench-agentic

ZeroBench solved by an **external tool-using agent** (e.g. Claude Code) — the
tool-assisted configuration of ZeroBench (ICML 2026). Same 100-question main set
as [`zerobench`](../zerobench/), which instead runs the upstream model-only
harness and makes its own LLM calls. Maps to the leaderboard's
externally-reported "Claude Code harness tools" (tool-assisted) entries.

**On the names:** `-agentic` marks the variant an *external agent* drives. It is
the plain `zerobench` that is `eval.benchmark.agent="native"` — "native" meaning
the benchmark ships its own agent scaffold (`benchmarks/RULES.md` 12a) and calls
the model itself. So the suffix marks which side supplies the agent, not which
one has an agent at all.

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
| Paper | [arXiv:2502.09696](https://arxiv.org/abs/2502.09696) |
| Leaderboard | [zerobench.github.io](https://zerobench.github.io/#leaderboards) |
| Dataset revision | `8fcdfd9138c802cac52b774b36946e70f738f500` |

## Model-only vs agent — two benchmarks, one scorer

- **`zerobench`** (native harness) — model-only, "no agent" (the `native` agent).
- **`zerobench-agentic`** (this) — agent-facing; the agent sees the question + its
  1–3 image files in `/app` and prints a deliberate final line
  `FINAL ANSWER: <answer>`.

Both score through the **same shared module** `core/test-answer-match`
(`score_answer.py`) — one `is_correct` (case-insensitive match), with
**surface-appropriate extraction**: the native path takes the last `{...}` from a
single model completion (upstream's protocol); the agent path takes the last
`FINAL ANSWER:` sentinel line, because an agent *transcript* has incidental
braces and can append tool output after the answer, so "last `{...}`" is unsafe
there. The native harness imports `extract_answer`/`is_correct`; this benchmark runs
its `/test.sh` as `/grade.sh`. One implementation, no drift.

## Gated data

`jonathan-roberts1/zerobench` is license-gated on HuggingFace; the build fetches
it via an ephemeral `--mount=type=secret,id=HF_TOKEN` (benchmarks/RULES.md 8a).
See [`tests/build/known-broken.md`](../../../tests/build/known-broken.md).

## Files

- `Dockerfile` — gated fetch → agent-facing task (images in `/app`) + shared grader
- `compose.yaml` — compose runner for `eval-containers run zerobench-agentic`
- `docker-bake.hcl` — bake target `benchmark-zerobench-agentic`
- `README.md` / `AUDIT.md`
