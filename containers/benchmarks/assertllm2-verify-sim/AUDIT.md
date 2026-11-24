---
benchmark: assertllm2-verify-sim
host: local podman (macOS, linux/arm64, native)
commit: feat/751-assertllm2-verify
---
# Audit — assertllm2-verify-sim (AssertLLM2, 3 designs / 19 tasks)

Status: `✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build` of `core/sby`, `core/assertllm2-harness` and this image on linux/arm64; the harness's build runs `build-tasks.py` and `check-tasks.py` and fails unless the tasks that pass are exactly `tasks.txt` (19). Not yet built for linux/amd64 |
| running | ? | not run with a live agent (oracle and grader checks only) |
| isolation | ✓ | in the built image, as uid 1002 (`setpriv`): `cat /tasks/all.jsonl`, `ls /tasks`, `ls /tasks/1`, `cat /tasks/1/label.txt`, `ls /tasks/build`, `ls /tests`, `cat /tests/grade.py` all denied. `/app` holds only `spec.md`, `figures/` and `rtl/`, every entry dated 2000-01-01; no other design RTL on the filesystem; the prompt names the top module, never the variant; the task id is withheld by the framework (`env -i`, rule 7). Within each design, files outside the regenerated set are byte-identical across variants, and no path or file says mutant, merged or bug (checked over all 128 candidate tasks of the 19 checked designs) |
| oracle | ✓ | `eval-containers oracle assertllm2-verify-sim --task-id <t>`: gold=1.0, no-op=0.0 for task 0 (golden), 1 (individual mutant) and 6 (merged). Over all 19 tasks, through `/grade.sh`: the derived verdict scores 1.0 with any, an empty or no explanation; the opposite verdict, malformed JSON, no answer and no verdict score 0.0; always-`correct` = 0.158 raw / 0.50 balanced, always-`incorrect` = 0.842 raw / 0.50 balanced. The derived verdict checks the golden/mutant assumption, not the spec |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | n/a | upstream publishes assertion-generation scores (JasperGold), not verdicts; nothing to replicate |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `eval.benchmark.internet="false"` and `ENV EVAL_INTERNET=false`; `compose.yaml` extends the shared runner (internal network only); grading is offline |
| agent-nonroot | ✓ | the shared runner runs the agent as `gosu agent`; `entrypoint.sh` only hands `/app` to uid 1002; `grade.sh` reads the answer as uid 1002, so a symlink into `/tasks` reads nothing (checked: scores 0.0) |
| secrets-isolated | ✓ | no credential at build or run: the upstream tarball, PyPI and Debian are public; no `ARG`/`ENV` secret |
| resource-limited | ✓ | 4 CPU / 8G in `compose.yaml` and `_chart/presets/assertllm2-verify-sim.yaml` (4 / 8Gi), rule 24e — declared, not yet measured under an agent |

## Size

| Metric | Value |
|--------|-------|
| image | 863 MB (`docker image ls`): the harness's 364 MB plus iverilog, verilator, make and g++ |
| per-task multiplier | 1 (shared-env) |

## Speed

| Metric | Value |
|--------|-------|
| build | harness 57 s with apt/pip layers cached (19 tasks; the 128-task check of all 19 checked designs took about 14 min on 4 CPUs); this image 85 s on top of the harness (apt) |
| grade | under 1 s (`grade.py`, stdlib) |
| end-to-end | ? (no agent run); one oracle run (fetch, rebuild one design, compare, grade, twice) 150–210 s |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (no live agent run yet) |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks (`docker run` of the tag: registry denied) |
| pull size | — | not published |

Tools in the image: `iverilog`, `verilator` and `g++` present; no yosys, sby or z3.
