# native

Not an agent: the marker that a benchmark brings its own. A benchmark whose
upstream harness is the agent declares `LABEL eval.benchmark.agent="native"` and
ships the harness at `/harness.sh`; this image is the only agent it pairs with,
so its eval image is `evals/<benchmark>--native`. `/run.sh` execs `/harness.sh`,
and `/opt/agent/native` makes `core/runner/run-agent` launch it as root with the
task identity instead of through the scrubbed agent phase
(`.agents/benchmarks/RULES.md` 12a–12d, `.agents/agents/RULES.md` 23).
