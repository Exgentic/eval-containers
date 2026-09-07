# hwe-bench

HWE-Bench — real fail-to-pass RTL bug-repair tasks graded by simulation ("SWE-bench for hardware").

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 169 (5 projects) |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [github.com/pku-liang/hwe-bench](https://github.com/pku-liang/hwe-bench) |
| Dataset | [henryen/hwe-bench](https://huggingface.co/datasets/henryen/hwe-bench) |
| Paper | [arXiv:2604.14709](https://arxiv.org/abs/2604.14709) |
| Dataset revision | `82a42e0a05719366a326e09ddc668ea0d46c91f6` |

Covers the five source-buildable HWE-Bench projects; the same simulation grader
handles all of them (it just runs each case's self-contained `tb_script` and reads
the markers, so it is agnostic to the underlying toolchain):

| Project | HDL / flow | Sim | Cases |
|---------|-----------|-----|------:|
| lowRISC/ibex | SystemVerilog | Verilator | 35 |
| openhwgroup/cva6 | SystemVerilog | Spike (ISA) | 35 |
| chipsalliance/caliptra-rtl | SystemVerilog | Verilator | 16 |
| chipsalliance/rocket-chip | Chisel → Verilog | Mill + Verilator | 31 |
| OpenXiangShan/XiangShan | Chisel → Verilog | Mill + Verilator | 52 |
| **total** | | | **169** |

**OpenTitan is out of scope** — its 245 cases need commercial Synopsys VCS and its
images are not distributed. It is 59% of the full 417-case benchmark, so 172 cases
are runnable with open tooling.

**3 of those 172 are excluded** because their gold fix lives entirely or partly in a
git submodule (a `Subproject commit` pointer bump), which the grader cannot score
under this benchmark's contract that the agent edits only tracked superproject HDL
source: `chipsalliance__rocket-chip-177` (fix is *only* a submodule bump — unsolvable
by editing the superproject), and `openxiangshan__xiangshan-2246` /
`openxiangshan__xiangshan-2781` (mixed submodule + source, where the source hunk
alone does not make the failing test pass — verified: gold scores 0). The remaining
11 mixed submodule+source cases are kept: their source hunk alone resolves the test
(gold scores 1), and both the grader (`grade.sh`) and the oracle (`solution.sh`)
ignore submodule-pointer hunks so those cases grade correctly. Net: **169 gradeable**.

## What the agent sees

The agent receives a task of the form: "Fix the hardware bug in the repository at
`/home/<repo>`. Edit the design source (SystemVerilog/Verilog RTL, or Chisel/Scala
for the Chisel-based repos) so the failing test passes. Do NOT modify any test files
or testbenches." The problem text is read from
`/tasks/0/problem.txt` (the dataset's `problem_statement`) and passed in via the
`TASK` environment variable. Because this benchmark uses a per-task environment,
each task builds a separate image; the agent works inside the checked-out RTL repo
(already at the buggy base commit, `chown`ed to `agent`) and edits files in place.

## How it's graded

HWE-Bench's own **deterministic, simulation-based** method — no LLM judge. Each
case carries a self-contained `tb_script` that generates its testbench, drives the
repo through its own simulation flow (Verilator for the SystemVerilog repos, Spike
for cva6, and Mill-based Chisel elaboration → Verilator for the Chisel repos — all
baked into the base image), and prints `TEST: <name> ... PASS|FAIL|SKIP` markers
between `HWE_BENCH_RESULTS_START/END`. The grader never invokes any of these tools
itself; it runs `tb_script` and parses the markers, so it is toolchain-agnostic.

`grade.sh` captures the agent's diff (`git diff` → `/home/fix.patch`), then
`grade.py` resets the repo to the buggy base commit, applies that diff, runs
`tb_script`, and writes `1` to `/logs/verifier/reward.txt` iff every fail-to-pass
test PASSES (else `0`). Strict resolved@1 — the leaderboard metric; no fractional
reward. Grading is fully offline (Verilator baked into the base image; no dataset
package at run time).

## Per-task build

`env=per-task`. Unlike swe-bench-pro, the base image ref needs **no** dataset
lookup — it is a pure string transform of the task id:

| | |
|---|---|
| task id | `<org>__<repo>-<number>` **lowercase** (e.g. `lowrisc__ibex-2232`) |
| base image | `ghcr.io/pku-liang/<org>_m_<repo>:pr-<number>` (org/repo lowercased) |
| repo path | `/home/<repo>` lowercased (e.g. `/home/ibex`, `/home/xiangshan`) |

The id splits on the **last** `-` for `<number>` and on `__` for `<org>`/`<repo>`,
so hyphenated repo names (`caliptra-rtl`, `rocket-chip`) parse correctly.

Task ids are **lowercase** so they are Docker-safe: the id is the per-task
image-name namespace, and `compose.yaml` interpolates the raw `${EVAL_TASK_ID}`
into the runner image ref (YAML can't lowercase). The dataset's per-repo JSONL
files keep the upstream mixed case (`lowRISC__ibex.jsonl`,
`OpenXiangShan__XiangShan.jsonl`), so `build.sh` maps the lowercase orgrepo back
to that filename (only `lowRISC` and `OpenXiangShan` differ) and passes it to the
Dockerfile as `HWE_HF_SLUG`.

`build.sh` derives all three from the id and `docker build`s the overlay
Dockerfile `FROM` the pku-liang per-PR image, fetching the case's JSONL row from
the pinned dataset revision to bake the problem statement, gold `fix_patch`, base
commit, `tb_script`, and fail-to-pass list root-only.

## Files

- `Dockerfile` — overlay on the pku-liang per-PR image
- `build.sh` — per-task PULL + overlay (derives base ref + repo path from the id)
- `grade.py` / `grade.sh` — HWE's simulation grader
- `entrypoint.sh` — seeds `TASK`, hands the repo to the agent
- `solution.sh` — oracle gold (applies the dataset `fix_patch`)
- `compose.yaml` / `docker-bake.hcl` — run + bake wiring
- `tasks.txt` — the 169 task ids, grouped by project (documentation only; not parsed by the harness)
- `README.md` / `AUDIT.md` — this file + the audit report
