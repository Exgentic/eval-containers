# mock

A deterministic agent. It reads `$TASK`, writes a fixed answer to stdout, a line
to stderr, and exits 0 — no model, no network, no state.

It exists so a test can assert what the machinery *around* an agent produces:
`task/result.json`, `agent/result.json` (including the exit code),
`model/result.json`, the captured stdout/stderr, and the run's directory layout.
Every other agent's output depends on a model, so a run of one can only show that
something happened, not that each piece was written.

Paired with `benchmarks/agents-smoke` — a ~33 MB benchmark with an
unconditional-pass grader — `evals/agents-smoke--mock` is the smallest complete
eval in the fleet: `tests/e2e/cluster-contract.sh` runs it on a kind cluster, and
it doubles as a zero-cost way to check that a deployment can run anything at all.

Not a substitute for the replay path: gateway and trace coverage lives in
`models/replay` and `tests/run/replay`, where a real agent makes real calls
against recorded traffic.
