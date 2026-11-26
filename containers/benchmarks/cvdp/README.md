# cvdp

CVDP (Comprehensive Verilog Design Problems) — **single-turn** Verilog design,
modification, lint/QoR and debug tasks: the model answers in one call with
upstream's own prompt, and the files it returns are graded by CVDP's own
cocotb/Yosys harnesses on open-source tools. Model-only (native harness).
Sibling of the agent-driven [`cvdp-agentic`](../cvdp-agentic/README.md); both
share `core/cvdp-sim`.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 302 |
| Environment | shared-env |
| Agent | `native` (benchmarks rule 12a) |
| Internet required | false |
| Released | no |
| Upstream | [github.com/NVlabs/cvdp_benchmark](https://github.com/NVlabs/cvdp_benchmark) |
| Dataset | [nvidia/cvdp-benchmark-dataset](https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset) |
| Paper | [arXiv:2506.14074](https://arxiv.org/abs/2506.14074) |
| Dataset revision | `5b807d945f6a99aa645f7e43a64a2115e281b4bf` |
| Harness revision | `8e894cf74414ab1eaea1e2b4e80a02f123df07b6` |

## Scope

The 302 tasks of CVDP v1.1.0's `nonagentic_code_generation_no_commercial` split:

| Category | Task | Tasks |
|----------|------|------:|
| cid002 | RTL code completion | 94 |
| cid003 | Natural-language spec → RTL | 78 |
| cid004 | Modify existing RTL | 55 |
| cid007 | Lint / QoR improvement (25 graded partly by Yosys synthesis) | 40 |
| cid016 | Debug / fix RTL | 35 |

Out of scope: the verification categories (cid012/013/014 — Cadence Xcelium
only, `commercial` split) and the code-comprehension split (cid006/008-010 —
scored by BLEU and an LLM judge, not by simulation).

## How a task runs

`/harness.sh` (`run_cvdp.py`) reproduces upstream's non-agentic flow
(`src/dataset_processor.py`, `src/model_helpers.py`, `src/llm_lib/openai_llm.py`):

1. **Prompt.** System: upstream's folder-structure context + the category's
   guidance line (+ the JSON response schema when the task expects several
   files). User: each input file in a fenced block, then
   `Provide me one answer for this request: <prompt>`, then where the answer goes
   (`…saved directly to: rtl/x.sv` for one file, `Name the files as: [...]` for
   several).
2. **Call.** One `chat/completions` request through the gateway, 60 s timeout.
3. **Parse.** One output file → the first fenced code block, else the whole
   reply. Several → the `{"code": [{"<name>": "<code>"}]}` object. An
   unparsable reply is retried up to 3 times, as upstream.
4. **Grade.** The input files plus the returned files are laid out under
   `/code`, and the task's own hidden harness runs unmodified via the shared
   `/tests/cvdp_grade.py` (same mechanics as `cvdp-agentic`: each compose service
   in `/code/rundir` with its `env_file`, pass iff every service exits 0).

Reward is `1.0` or `0.0`. `/output/task/cvdp.json` records the parse attempts,
the files written, and the grader's per-service exit codes; the per-service
pytest logs are under `/output/task/verifier/services/`.

### Known upstream harness issues

Reported against v1.1.0 on the upstream tracker; scores here inherit them:
[#60](https://github.com/NVlabs/cvdp_benchmark/issues/60) (four cid003 harnesses
pass designs that ignore their inputs: factorial, complex_multiplier, dbi,
fibonacci_series) and
[#62](https://github.com/NVlabs/cvdp_benchmark/issues/62) (nine problems whose
hidden tests contradict their prompts).

## Oracle

The public dataset **redacts the reference solutions** (`output.context` is empty
for all 302 rows); upstream shares them with members of the
[Si2 LLM Benchmarking Coalition](https://si2.org/llm-benchmarking-coalition/).
`solution.sh` grades the reference files when present and fails loudly
otherwise, so on this revision only the **no-op** half of the oracle runs.

## Files

- `Dockerfile` — `FROM core/cvdp-sim` (simulator + shared grader), materializes the pinned rows, installs the harness.
- `materialize.py` — splits the JSONL into `/tasks/<n>/{id.txt,task.json}` and `harness/` (+ `output/` when present).
- `run_cvdp.py` — the native harness (prompt → model → parse → grade).
- `solution.sh` — oracle gold (grades the reference output; mounted, never baked).
- `compose.yaml` — compose surface (`runner-native`); k8s uses the shared chart (`presets/cvdp.yaml` pins `agent: native`).

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`): `BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running

```bash
eval-containers run cvdp --agent native --model <model> --task-id 0 --local --mode compose
```

Task `<n>` is line `<n>` of the pinned JSONL; `/tasks/<n>/id.txt` holds the
upstream id.
