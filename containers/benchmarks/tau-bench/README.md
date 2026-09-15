# tau-bench

TAU-bench - Tool-Agent-User interaction (retail + airline)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 165 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/sierra-research/tau-bench](https://github.com/sierra-research/tau-bench) |
| Paper | [paper](https://arxiv.org/abs/2406.12045) |
| Dataset revision | `59a200c6d575d595120f1cb70fea53cef0632f6b` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

There is no fleet agent, though: tau-bench's own runtime (the `harness` sidecar) drives the tool loop and the user simulator, and the runner's `/harness.sh` is the pass-through that answers its completion requests through the edge. The benchmark declares `LABEL eval.benchmark.agent="native"`, so it pairs only with the `native` agent and its eval image is `evals/tau-bench--native` (rule 12b).

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run tau-bench` (the runner extends the shared `compose/runner-native.yaml`; bridge, user-gateway and harness are bespoke sidecars)
- `benchmarks/_chart/presets/tau-bench.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=tau-bench`
- `README.md` — this file
