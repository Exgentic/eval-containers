# Live sweep test rules

The live category runs **real evaluations against real LLM providers**
across the full fleet. This is the gate that answers "does every
benchmark actually produce a trajectory end-to-end with a real model?"
Runs only in release verification.

Parent: [../RULES.md](../RULES.md)

## Scope

1. **Release verification only.** Live tests are `#[ignore]` by default
   and require the release runner's API credentials. They MUST NOT run
   in contribution verification.

2. **The output is the fixture set.** Every live-sweep run produces
   trajectories that feed contribution-verification replay. Every
   trajectory that passes the mechanical rules (parent rule 13) is
   promoted to `tests/replay/fixtures/` with a provenance entry.

## Matrix

3. **Every buildable benchmark.** The sweep MUST attempt every
   benchmark that is NOT in `tests/build/known-broken.md` AND is NOT
   per-task-build. That's the maximal set the release runner can
   actually execute.

4. **≥3 tasks per benchmark.** Task 0, task N/2, and task N-1. Three
   tasks catches indexing bugs, JSONL-row-off-by-one errors, and
   materialize-helper edge cases that a single task 0 run misses.

5. **Model of record: gpt-5.4.** Selected for price/speed/reliability.
   Changing the model of record MUST be a documented decision in the
   top-level `RULES.md` changelog — it invalidates every existing
   fixture.

6. **Agent: claude-code.** The reference agent. Adding agents to the
   live sweep multiplies runs by N_agents; each additional agent MUST
   be justified by a concrete signal it produces that claude-code does
   not (e.g. different tool-use surface).

## Budget

7. **Respect a budget cap.** The sweep driver MUST accept a
   `DOCK_LIVE_BUDGET_USD` env var and halt if projected cost exceeds
   it. Estimated cost is the sum of `total_cost` from each run's
   `/output/model/result.json`.

8. **Halt on unrecoverable failure.** A failure to start the compose
   stack, a model 401/403, or an infra error halts the sweep. Budget
   MUST NOT be spent on retries of infra failures.

## Trajectory inspection

9. **Every live run MUST pass the trajectory rule catalog.** After each
   run, the driver MUST invoke
   `tests/sanity/test.rs::inspect_trajectory()` against the fresh
   `trajectory.jsonl`. Any red rule (refusal, max-tokens truncation,
   no substantive output, wrong-answer format) blocks promotion of
   that fixture.

10. **Yellow rules annotate the fixture, not block it.** A yellow
    finding (e.g. moderate retry count, one content filter warning)
    is recorded in the provenance entry but the fixture is still
    promoted — yellows are informational.

## Fixture promotion

11. **Pass fixture → `tests/replay/fixtures/`.** Rename
    `output/<bench>/<task>/model/trajectory.jsonl` →
    `tests/replay/fixtures/<bench>-<task>-claude-code.trajectory.jsonl`
    and add an entry to `fixtures/provenance.json`. The promotion is
    a single commit for auditability.

12. **Fail fixture → `known-broken.md`.** A run that fails an
    inspection rule is not promoted to fixtures. Instead, its failure
    is recorded in `tests/live/known-broken.md` with the specific rule
    that tripped and a citation to the run output. The next release
    cycle re-attempts it.
