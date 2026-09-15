# native

Not an agent: the marker that a benchmark brings its own. A native-harness
benchmark — one whose upstream harness is the agent scaffold (tau-bench, walle,
automationbench) — declares `LABEL eval.benchmark.agent="native"` and ships the
harness at `/harness.sh`. This image is the only agent it pairs with, so its
eval image is `evals/<benchmark>--native`: one image, named and built like every
other, instead of one per agent that never runs.

`/run.sh` execs `/harness.sh`. `/opt/agent/native` tells the shared launcher
(`core/runner/run-agent`) to run it as root with the task identity — the harness
resolves the task and grades in-process, the way a verifier does — instead of
through the scrubbed, unprivileged agent phase. Everything else is the standard
pipeline: the edge records every model call, the verifier and result writer run
after it, on all three surfaces.

Pairing is structural, not a choice: `eval-containers run <native benchmark>`
resolves `--agent native` and refuses any other, and `--agent native` is refused
on a benchmark that does not declare it (`.agents/benchmarks/RULES.md` 12b,
`.agents/agents/RULES.md` 23).
