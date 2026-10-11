# swe-bench-multimodal

SWE-bench Multimodal - fixing visual front-end bugs from issues with screenshots

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 480 (the `test` split) |
| Environment | per-task |
| Platform | linux/amd64 only |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/SWE-bench/SWE-bench](https://github.com/SWE-bench/SWE-bench) |
| Dataset | [SWE-bench/SWE-bench_Multimodal](https://huggingface.co/datasets/SWE-bench/SWE-bench_Multimodal) |
| Paper | [paper](https://arxiv.org/abs/2410.03859) |
| Dataset revision | `4e6662d51c48e475f7f346e4fa09a6f8b31fcaa5` |

Real GitHub issues from 11 JavaScript projects whose bugs are visual — UI
component libraries (carbon, next, grommet), map and diagram rendering
(openlayers, bpmn-js), syntax highlighting (prism, highlight.js), and tooling
around them (lighthouse, eslint, prettier, scratch-gui). 434 of the 480 issues
embed at least one screenshot.

## Task ids

A task id is the tag upstream publishes that instance's image under: the
instance id lowercased, with `__` spelled `_1776_`. `openlayers__openlayers-12683`
is task `openlayers_1776_openlayers-12683`. [`tasks.txt`](tasks.txt) lists all 480;
`/tasks/0/id.txt` in each image holds the original instance id.

## What the agent sees

"Fix this GitHub issue in the repository at /testbed. Edit the source code to
resolve the bug. Do NOT modify any test files.", followed by the issue text, in
the `TASK` environment variable.

Two things the prompt adds, because the agent could not find them otherwise:

- **The screenshots are files.** The agent has no internet, so the issue's image
  links are dead to it. Upstream archives what each issue links to inside the
  image, at `/problem_assets/`, and the prompt says so.
- **The project's Node.js is not on `PATH`.** The agent runs with a scrubbed
  environment, and a Node-based agent brings its own `node`, which is rarely the
  version the project needs (they range from v4 to v21). The prompt names the
  one command that loads the right one: `source /usr/local/nvm/nvm.sh`.

## How it's graded

`/grade.sh` runs after the agent exits, in the same container, and reuses the
**official SWE-bench harness** (`swebench==5.0.2`) for everything that decides
the score:

1. **Run**: `/tasks/0/eval.sh`, the instance's own eval script from the dataset,
   as the harness prepares it — it resets the test files, applies the
   `test_patch`, and runs the project's tests.
2. **Grade**: `/tests/grade.py` scores the log with the harness's
   `get_eval_report` — the project's log parser, then `FAIL_TO_PASS` /
   `PASS_TO_PASS` under the instance's `eval_type`. Reward is `1` iff the
   instance resolves.

Two differences from running the harness itself, both because grading happens
inside the task's own container:

- The eval script runs here, on the tree the agent edited, where the harness
  would start a fresh container and apply the agent's patch to it.
- The binary test baselines (expected renderings, which a text patch cannot
  carry) are baked into the image at build time. The harness downloads them
  while grading; the commands that restore them are the harness's own.

The grader brings its own Python (`/tests/python`): upstream's images do not all
have `pip` to install the harness with, and nothing is installed into the testbed.

## What is hidden from the agent

`/tests` (the grader), `/tasks` (the dataset row with the gold patch, the eval
script, the baselines) and `/swebench` are root-only. `/swebench` is upstream's
own copy of the test baselines, which its images ship world-readable.

## Per-task build

`env=per-task`: the Dockerfile takes `EVAL_TASK_ID` and builds on
`docker.io/swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest`.

```bash
eval-containers build bench swe-bench-multimodal --task-id googlechrome_1776_lighthouse-10176
eval-containers oracle swe-bench-multimodal --task-id googlechrome_1776_lighthouse-10176 --platform linux/amd64
```

## Known limitations

Evidence for each is in [`AUDIT.md`](AUDIT.md).

- **Some tests need the network while grading.** carbon's accessibility tests
  download IBM's rule archive when they run, and the eval script re-installs
  dependencies when a change touches `package.json` (15 gold patches do). The
  verifier has a route on Kubernetes and in the standalone bundle; under
  `compose` the runner has none, so there a correct carbon fix can grade 0 and a
  `package.json` change waits out a 15-minute install timeout first.
- **An unchanged tree scored 1 on two of five openlayers tasks**, both under
  amd64 emulation on Apple Silicon. On one, Chrome has no WebGL there, the suite
  aborts at a WebGL test, and `fail_only` grading counts the tests that never ran
  as not failing. On the other the suite ran to the end and the listed
  `FAIL_TO_PASS` test simply passes before the fix — as it would for 45 more
  openlayers tasks that list the same test. Neither is checked on a native amd64
  host yet.
- **Not run with a live agent yet**, and not published: the release workflow does
  not enumerate this benchmark.

## Files

- `Dockerfile` — builds the benchmark image
- `materialize.py` — build time: writes the task record from the pinned dataset
- `grade.sh`, `grade.py` — the verifier
- `entrypoint.sh` — hands the repo to the agent and builds `TASK`
- `solution.sh` — the oracle's gold solution
- `tasks.txt` — the 480 task ids
- `compose.yaml` — compose file for `eval-containers run swe-bench-multimodal`
