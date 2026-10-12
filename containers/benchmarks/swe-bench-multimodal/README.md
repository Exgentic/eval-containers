# swe-bench-multimodal

SWE-bench Multimodal - fixing visual front-end bugs from issues with screenshots

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 395 of the 480 in the `test` split ([which ones](#which-tasks-ship)) |
| Environment | per-task |
| Platform | linux/amd64 only |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/SWE-bench/SWE-bench](https://github.com/SWE-bench/SWE-bench) |
| Dataset | [SWE-bench/SWE-bench_Multimodal](https://huggingface.co/datasets/SWE-bench/SWE-bench_Multimodal) |
| Paper | [paper](https://arxiv.org/abs/2410.03859) |
| Dataset revision | `4e6662d51c48e475f7f346e4fa09a6f8b31fcaa5` |

Real GitHub issues from 11 JavaScript projects whose bugs are visual — UI
component libraries (carbon, next, grommet), map and diagram rendering
(openlayers, bpmn-js), syntax highlighting (prism, highlight.js), and tooling
around them (lighthouse, eslint, prettier, scratch-gui). 386 of the 395 issues
that ship embed at least one screenshot.

## Task ids

A task id is the tag upstream publishes that instance's image under: the
instance id lowercased, with `__` spelled `_1776_`. `openlayers__openlayers-12683`
is task `openlayers_1776_openlayers-12683`. [`tasks.txt`](tasks.txt) lists the
395 that ship; `/tasks/0/id.txt` in each image holds the original instance id.

## Which tasks ship

A task ships only if, on native amd64, its reference fix scores 1 **and** the
unchanged repository scores 0. Every one of the 480 was run both ways; 395
pass.

59 are left out because the unchanged repository already scores 1 — a
task anyone solves by doing nothing. All 59 are openlayers. Most list a
`FAIL_TO_PASS` test that their own test patch does not touch and that passes
before the fix (`icon-opacity`, `full-build`); under the instance's `fail_only`
grading that resolves the task with no change, under the official harness too.

26 more, all carbon, are left out because their tests call an IBM endpoint that
no longer
answers as it did. The test command crashes on the reply, and the reference fix
scores 0, or 1 on one run and 0 on the next; a task whose log shows that crash is
left out even where the score held.

The ids and the per-project counts are in [`AUDIT.md`](AUDIT.md).

## What the agent sees

"Fix this GitHub issue in the repository at /testbed. Edit the source code to
resolve the bug. Do NOT modify any test files.", then the issue text, in the
`TASK` environment variable.

Upstream ships no prompt, only the issue, so every line around it is ours. It
states facts the agent cannot find out for itself and nothing about how to
work:

- **Where the screenshots are.** The agent has no internet, so the issue's image
  links are dead to it. Upstream archives what each issue links to inside the
  image, under `/problem_assets/`; each image link in the issue text is replaced
  by the path of its archived file, so the issue itself says which file shows
  what.
- **How to load the project's Node.js.** The agent runs with a scrubbed
  environment, and a Node-based agent brings its own `node`, which is rarely the
  version the project needs (they range from v4 to v21):
  `source /usr/local/nvm/nvm.sh`.
- **That Chrome is installed** (`google-chrome`), which is how a change can be
  looked at.
- **The container's real CPU and memory limit**, read from its cgroup. A
  container reports its host's CPUs, and tools such as jest size their worker
  pools from that: on a 96-core node one test run started 95 workers under a
  2-CPU, 12 GB limit and the whole task was OOM-killed.

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
eval-containers build bench swe-bench-multimodal --task-id
googlechrome_1776_lighthouse-10176
eval-containers oracle swe-bench-multimodal --task-id
googlechrome_1776_lighthouse-10176 --platform linux/amd64
```

## Known limitations

Evidence for each is in [`AUDIT.md`](AUDIT.md).

- **Grading needs a network route for some tasks.** carbon's accessibility tests
  download IBM's rule archive while they run, and the eval script re-installs
  dependencies when a change touches `package.json`. The verifier has a route on
  Kubernetes and in the standalone bundle, where every task above was checked.
  Under `compose` the runner sits on the internal network and has none:
  there a correct fix scores 0 on 13 of the 52 carbon tasks checked, and a tree
  whose `package.json` differs from `HEAD` waits out a 15-minute install timeout
  first.
- **Chrome has no WebGL.** Not under emulation and not on native amd64, with or
  without the sandbox, so an agent cannot render a WebGL layer to look at it.
  Grading is unaffected for the tasks that ship: each was checked with the
  reference fix and without it.
- **An unfixed tree can hang the tests.** On 3 of the tasks that ship the tests
  never finish while the bug is there (`openlayers-13020`, `openlayers-13212`,
  `openlayers-15271`). The verifier stops the
  tests after 30 minutes — the official harness's own limit — and scores 0.
- **A task that fails rarely may remain.** The check ran each reference fix
  once, and a second time on 60 of the 395 tasks. The second run is
  what exposed the carbon tasks above that score 1 on one run and 0 on the next.

## Files

- `Dockerfile` — builds the benchmark image
- `materialize.py` — build time, in the task's image: writes the task record
  from the pinned dataset
- `grade.sh`, `grade.py` — the verifier
- `entrypoint.sh` — hands the repo to the agent and builds `TASK`
- `solution.sh` — the oracle's gold solution
- `tasks.txt` — the 395 task ids that ship
- `compose.yaml` — compose file for `eval-containers run swe-bench-multimodal`
