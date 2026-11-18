# assertllm2-verify

AssertLLM2-verify - judge RTL against its spec, from the source alone

The agent gets a hardware design's specification and its complete Verilog RTL,
and says whether the RTL implements the specification **correctly** or
**incorrectly**, with an explanation for a person to read. Only the verdict is
scored.

There are three setups. They share one task set, the same labels and the same
grader (`core/assertllm2-harness`), and differ only in the tools the agent gets:

| Benchmark | Tools |
|-----------|-------|
| `assertllm2-verify` | none beyond the shell and Python: the spec and the source |
| [`assertllm2-verify-sim`](../assertllm2-verify-sim/) | Icarus Verilog and Verilator, for the agent's own testbenches |
| [`assertllm2-verify-formal`](../assertllm2-verify-formal/) | Yosys, SymbiYosys and Z3, for properties the agent model-checks |

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 19 (3 designs: 3 golden, 14 individual mutants, 2 merged) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/hkust-zhiyao/AssertLLM2](https://github.com/hkust-zhiyao/AssertLLM2) |
| Paper | [paper](https://arxiv.org/abs/2605.27472) (assertion generation; this reuses its designs) |
| Dataset revision | `9c481ca98fa0a78f248fcf2022127ab80f1e02df` |

Tracking issue: #751.

## The tasks

Drawn from upstream's 25-design **Lite** subset. 20 of its designs may be
redistributed here (`core/assertllm2-harness/designs.txt` names each one's
license and why each other one was left out). **Three are enabled so far** —
`ima_adpcm_encoder`, `srdy_drdy_library` and `bubblesortmodule` — to get the
benchmark working end to end before the rest follow. For every enabled design
the task set has:

- the **golden** RTL, labelled `correct`;
- **five individual mutants**, chosen from upstream's twenty with a fixed seed
  per design, labelled `incorrect` — "individual" rather than "single-bug",
  because upstream's records of a mutant's edits are not always complete;
- the **merged** variant, upstream's golden with several mutations at once,
  labelled `incorrect`.

A variant is a task only if it can be put in the common layout (below) and
passes every build check; `core/assertllm2-harness/tasks.txt` lists the tasks,
in order, and the build fails if that list changes. Two of bubblesortmodule's
variants fail a check (`tasks.txt` says which, and why), which leaves **19
tasks — 3 `correct`, 16 `incorrect`** (14 individual mutants, 2 merged).
Checking all 20 cleared designs found 14 usable, about 85 tasks; `tasks.txt`
records what failed and why, for when they are enabled.

`id.txt` holds `<design>/<variant>` (`bubblesortmodule/M_0004`); the agent never
sees it.

### The labels are assumed

**A golden is assumed correct and a mutant incorrect.** Nobody has checked
either against the specification: a golden may have a defect, and a mutant may
behave the same as its golden, or differ only where the specification says
nothing. The score is agreement with those assumed labels. The explanations an
agent writes are kept for exactly this: a person reading a confident "wrong"
verdict can find a bad label.

### Every variant looks alike

Upstream wrote each mutated file with PyVerilog's code generator: comments gone,
its own layout. Left like that, the one file in that style would itself say
"mutated". So for each design, every file that any of its variants changes is
written by that same emitter in **every** variant, the golden included, and
re-emitted until it stops changing — one pass writes `reg a, b;` as
`reg a;reg b;` and the next splits it, so a golden emitted once from source
would differ in layout from upstream's already-emitted mutants. The
golden's leading comment block (its license notice) on top; every other file is
the golden's, with line endings and trailing blanks normalized. The emitter is
upstream's own — PyVerilog 1.2.1 with upstream's patch, and its sanitizer — so
a design's variants differ only where they were mutated. Two of upstream's steps
are left out or undone because they break the RTL for every tool here: its
`translate_off`/`on` restorer places the pragmas by guesswork and can wrap whole
modules, so regenerated files carry none; and the generator writes an empty
statement as `@(*);`, which is put back as `;`. `build-tasks.py` says how; it
refuses a variant that comes out the same as its golden.

### One configuration

Preprocessing fixes every macro the RTL tests at what upstream builds it with:
nothing defined on the command line, since upstream's design configs name no
defines. A spec can still describe other configurations. srdy_drdy_library's
says `SDLIB_CLOCKING` and `SDLIB_DELAY` "can be overridden", and a golden that
cannot be overridden reads as wrong. So when a spec names a macro the RTL
tests, the task's spec ends with a *Configuration of the supplied RTL* section
listing each such macro's value in that build, as Icarus Verilog's preprocessor
resolves it. It is the same for every variant of a design. Of the enabled
designs only srdy_drdy_library has one.

### Every task builds

`check-tasks.py` lays each task out exactly as the agent will see it and runs
every setup's tools on it: Icarus Verilog (compile and elaborate), Verilator
(lint), Yosys (`prep`), and a depth-1 SymbiYosys BMC run with the one solver
shipped (smtbmc with Z3). A design whose golden fails is dropped whole, so every design
has both classes. That a task builds says nothing about whether its label is
right. The results, with each tool's version and command, stay in the image
(`/tasks/build/checks.jsonl`, root only).

## What the agent sees

`/app/spec.md` (upstream's curated specification) with its figures under
`/app/figures/`, and the RTL under `/app/rtl/`, with `/app/rtl/files.f` listing
the sources in compile order (`+incdir+include`). The prompt names the top
module and the setup's tools, and asks for `/home/agent/answer.json`:

```json
{"verdict": "incorrect", "explanation": "The spec requires ..., but ..."}
```

Every file in `/app` has the same timestamp. The other variants, the labels and
the grader are root-only (`/tasks`, `/tests`), and there is no internet.

## How it's graded

`/grade.sh` reads the answer as the agent's uid (so a symlink into `/tasks`
reads nothing) and runs `/tests/grade.py`. The reward is `1.0` when `verdict` is
exactly `correct` or `incorrect` and matches the label, and `0.0` otherwise —
including a missing answer or malformed JSON. The explanation never changes the
reward.

`/output/task/answer.json` keeps the answer as written, and
`/output/task/verifier_report.json` the verdict's details: `label`,
`variant_kind` (`golden`, `individual_mutant`, `merged`), `verdict`,
`answer_valid`, `verdict_correct` and `explanation_present`.

### Reading the scores

At 3 goldens to 16 mutants, answering `incorrect` every time scores about 84%
raw accuracy (16 of 19), and with three goldens the `correct`-class accuracy
moves in steps of a third: read this first set as a working check, not a
measurement. Report **balanced accuracy** — the mean of the accuracy
on `correct` tasks and on `incorrect` tasks, 0.5 for any constant answer — with
the per-class accuracies, the accuracy on individual and on merged mutants, and
each group's count. In one run's directory
(`<output-dir>/<benchmark>/<agent>/<model>/<run-id>/`):

```bash
for d in */; do
  jq -c --slurpfile a "$d/agent/result.json" '. + {run_error: $a[0].error}' \
    "$d/task/verifier_report.json"
done | jq -s '{failed_runs: map(select(.run_error)) | length,
  scored: (map(select(.run_error == null)) | group_by(.label)
    | map({label: .[0].label, n: length,
           accuracy: ((map(select(.verdict_correct)) | length) / length)}))}'
```

An invalid answer counts as wrong in its class. A run that errored (the agent
timed out or crashed: `agent/result.json` has an `error`) is still graded and
has a report, but it is not a score: count it under `failed_runs` and rerun it.

These are not upstream's numbers: upstream scores generated assertions with
JasperGold. Several variants of one design are not independent designs, and the
golden RTL is public, so a model may have seen it.

## Oracle

`solution.sh` (shared by the three setups) derives each verdict instead of
reading it: as root, with network, it fetches the pinned upstream tree, rebuilds
the task's design with the harness's own `build-tasks.py`, and answers `correct`
when the task's RTL is that design's golden and `incorrect` when it is not. That
checks the tasks and the grader against the golden/mutant assumption, not
against the specification.

```bash
eval-containers oracle assertllm2-verify --local --task-id 0
```

## Run

```bash
TASK_ID=0 docker compose -f containers/benchmarks/assertllm2-verify/compose.yaml up
```

Lean base: `BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/assertllm2-verify:latest`,
`AGENT_IMAGE=ghcr.io/exgentic/agents/<agent>:latest`, `AGENT_VERSION` per agent.
