# chipbench

ChipBench (UCSD / Columbia) — Verilog generation and debugging on realistic
modules: hierarchical designs, CPU IP units and injected-bug repair, graded by
simulation against the golden module. Model-only.

**Status:** Not yet released — no replay fixture landed (see
[`.agents/benchmarks/RULES.md`](../../../.agents/benchmarks/RULES.md) rule 21a).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 223 (45 generation + 89 zero-shot debugging + 89 one-shot debugging) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Evaluation | **model-only**, one call per task (no agent) |
| Upstream | [github.com/zhongkaiyu/ChipBench](https://github.com/zhongkaiyu/ChipBench) |
| Paper | [arXiv:2601.21448](https://arxiv.org/abs/2601.21448) |
| License | MIT |
| Dataset revision | `b4ba3a06759804ec88c26c3cf15b28b9a90b3443` |
| Agent | `native` — upstream's own single-call harness; the only pairing (rule 12b) |

## Tasks

Task ids are sequential; `id.txt` holds `<split>/<problem>`.

| Ids | Split | Count | What the model gets |
|-----|-------|------:|---------------------|
| 0–29 | `gen_self_contain` | 30 | spec of a stand-alone `TopModule` |
| 30–35 | `gen_not_self_contain` | 6 | spec + submodule code; write the hierarchical top |
| 36–44 | `gen_cpu_ip` | 9 | spec of a unit taken from an open-source CPU |
| 45–133 | `debug_zero_shot_{arithmetic,assignment,state_machine,timing}` | 24 / 30 / 6 / 29 | spec + buggy `TopModule` |
| 134–222 | `debug_one_shot_*` | 89 | the same, plus the bug's simulation waveform (`.vcd`) inline |

Problems are enumerated from the `*_prompt.txt` files: upstream's `problems.txt`
lists only 1 of 6 (`dataset_not_self_contain`) and 1 of 9 (`dataset_cpu_ip`)
problems at the pinned commit. The repo has 30 self-contained problems where the
paper (Table 1) counts 29, hence 45 generation tasks, not 44.

Out of scope: the reference-model track (Python / SystemC / CXXRTL). Upstream's
`Ref Model Gen/data.jsonl` at this commit is 100 CodeV training samples, not the
paper's 132 benchmark items, and the track is not wired into the harness.

## How a task runs

`/harness.sh` (`run_chipbench.py`) is a port of upstream's `make` flow (VerilogEval v2)
with `configure`'s defaults — the paper does not state other settings:

1. **Prompt** — `scripts/sv-generate --task=spec-to-rtl --examples=0`: system
   message *"You are a Verilog RTL designer that only writes code using correct
   Verilog syntax."*; user message `Question:` + `prompt.txt` + the
   `[BEGIN]`/`[DONE]` instruction + `Answer:`.
2. **Call** — one chat completion through the gateway, `temperature=0.85`,
   `top_p=0.95`, `max_tokens=16384`; a failed call is retried up to 10 times,
   20 s apart. If the provider rejects `top_p` / `temperature` with a 400, the
   harness drops that knob (sv-generate itself omits `top_p` for Claude, Gemini
   and GPT-5) and records it in the export.
3. **Extract** — sv-generate's parser: the `[BEGIN]`…`[DONE]` lines, else the
   first fenced block, else the text up to `endmodule`.
4. **Grade** — `grade_sv.py`: `iverilog -Wall -Winfloop -Wno-timescale -g2012 -s tb`
   on answer + `test.sv` + `ref.sv`, `timeout 30` simulation, then
   `sv-iv-analyze`'s verdict. **Reward 1.0** iff it is a pass (`.`): no
   compile error class matched and the testbench printed `Mismatches: 0`.

Icarus Verilog is v12 (`v12_0`), as in upstream's Dockerfile. `ref.sv` and
`test.sv` live root-only under `/tasks`; the model never executes code
(rule 12d).

**Output:** `/output/task/chipbench.json` — task id, split, sampling actually
sent, every call attempt, the raw response, the verdict code and the reward.
The extracted answer, simulator log and verdict are under
`/output/task/verifier/sim/`.

## Known upstream defects (10 tasks no answer can pass)

Found by the oracle sweep (gold 213/223). Kept as-is so scores stay comparable
with the paper, which graded the same files:

| Tasks | Cause |
|-------|-------|
| `*/Prob013_least_common_multiple` — ids 13, 53, 81, 116, 142, 170, 205 | the golden module drives `lcm_out = 'hz` while its GCD register is 0; the testbench compares `ref === (ref ^ dut ^ ref)`, and `z ^ z` is `x`, so even the golden module mismatches (389 / 399 samples) |
| `gen_not_self_contain/Prob006_cpu_top` — id 35 | `test.sv` line 223 (`initial stats1 = '{default:0};`) is a syntax error in Icarus v12 — the pinned tag and upstream's `v12-branch` head alike |
| `gen_cpu_ip/Prob008_chapter12_ctrl`, `Prob009_cp0_reg` — ids 43, 44 | the testbench never prints `Mismatches: N in M samples`, the only line `sv-iv-analyze` accepts as a pass |

Oracle note: the CPU-IP golden code uses `` `define `` macros that only `test.sv`
declares, and `test.sv` compiles after the answer, so `solution.sh` restates
them. A model answer is unaffected: the prompts never mention the macros.

## Replication (Claude 4.5 Opus, paper Tables 2 and 4)

`claude-opus-4-5-20251101` (the snapshot upstream's sv-generate names), ids 0–133
(generation + zero-shot debugging), 5 samples per task, unbiased pass@k averaged
per category and then across categories, as the paper does. The image takes the
upstream commit as a build arg (`--build-arg BENCHMARK_VERSION=<sha>`).

| Upstream commit | Gen pass@1 | Gen pass@5 | Debug pass@1 | Debug pass@5 |
|-----------------|-----------:|-----------:|-------------:|-------------:|
| paper (Table 2 / 4) | 30.74 | 38.52 | 47.45 | 60.04 |
| `2dfbb80` (initial commit, 2026-01-28 — the paper's) | 34.44 | **38.52** | 42.93 | 61.71 |
| `b4ba3a0` (pinned default, 2026-08-06) | 46.81 | 50.37 | 39.77 | 48.17 |

At the paper's commit the reproduction lands within sampling noise. The pinned
default scores higher on generation because upstream rewrote 23 generation
prompts and fixed several references and testbenches after publication.
At `2dfbb80` the oracle also finds `Prob007_data_accumulation_output` (assignment
bug, both shots) passing with the buggy code unchanged, which upstream fixed in
"Fix accumulation input handshake"; the pinned default has no such task.

## Run it

```bash
eval-containers run chipbench --agent native --model <model> --task-id 0
eval-containers oracle chipbench --task-id 0   # gold = RefModule renamed TopModule
```

Upstream reports pass@1/5/10 over n samples; one run here is one sample —
repeat with `-r` for pass@k.
