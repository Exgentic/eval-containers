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
   `EVAL_LIVE_BUDGET_USD` env var and halt if projected cost exceeds
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

## Trace inspection checklist

When a human or agent inspects a live run artifact at
`tests/live/runs/<bench>-<task>-<agent>/`, walk this checklist. Every
assumption the evaluation depends on MUST be verified against the
artifact, not assumed from "the run exited 0". The mechanical rule
catalog at [tests/sanity/task_inspection.rs](../sanity/task_inspection.rs)
implements roughly half of this; the other half currently requires
human or agent judgment until we get more mechanical coverage.

### A. Was the task actually delivered to the agent?

13. **Task prompt reached the agent.** `task/input/problem.txt` exists
    and is non-empty. `agent/stdout.log` or the first
    `trajectory.jsonl` request contains the problem text verbatim (not
    a truncated/mangled version). If the benchmark has attached files
    (images, code, data), they MUST be referenced in the trajectory.

14. **Ground truth was NOT in the agent's context.**
    `task/input/answer.txt` exists (it's what `test.sh` compares
    against) BUT MUST NOT appear anywhere in the agent-visible surface:
    not in `TASK`, not in any tool result, not in `stdout.log` before
    the agent's final answer. A leaked answer invalidates the run.

15. **System context matches the benchmark contract.** The first
    request in `trajectory.jsonl` should include the system prompt the
    benchmark expected. For example, a code-generation benchmark
    should set a coding-appropriate system message; an MCQ benchmark
    should include the "print only the letter" instruction.

### B. Did the model actually engage with the task?

16. **Non-empty substantive response.** At least one assistant message
    in `trajectory.jsonl` has non-empty `content` (covered mechanically
    by `no_substantive_output` in task_inspection.rs).

17. **No refusal.** The final assistant response is NOT a refusal or
    safety disclaimer. Mechanically checked by `refusal_final_response`
    and `content_filter_refusal`. Human spot-check: does the last
    message read like an answer, or like "I cannot help with that"?

18. **No `max_tokens` truncation.** No assistant message has
    `finish_reason == "length"`. Mechanical: `max_tokens_truncation`.
    Human: does the final answer look truncated mid-sentence?

19. **The model attempted the actual task.** The agent's reasoning
    (chain-of-thought, tool calls, or reply) should reference the
    *specific* problem from `task/input/problem.txt`. A run where the
    model restates a generic answer unrelated to the problem is red.

20. **No retry storm.** No more than 3 consecutive identical user
    prompts. Mechanical: `retry_storm`. A run that loops the same
    prompt 10 times is a harness bug, not a model answer.

### C. Did the model understand the environment?

21. **Tool availability was recognized.** If the benchmark exposes
    tools (filesystem, shell, browser, code interpreter), the model
    should either use them or explicitly reason about them. A run
    that reasons in pure prose while the benchmark hands it a
    `bash` tool is a red flag — the model didn't see the contract.

22. **Tool calls are well-formed.** If the model calls tools, every
    `tool_calls` entry has valid JSON arguments, and every call has
    a matching `tool_result` message in the next turn. No orphan
    tool calls, no argument parse errors.

23. **No phantom errors.** The model is NOT reporting errors that
    aren't real — e.g. "I don't have internet access" when the
    benchmark is a pure text MCQ, or "the file is missing" when the
    file is present in `task/input/`. Phantom errors indicate the
    model misjudged the environment.

24. **File context is accurate.** If the benchmark exposes files, the
    model's references to file content should match what's actually
    in `task/input/` — no hallucinated file contents, no
    misremembered filenames.

### D. Was the final answer well-formed?

25. **Format matches grader expectations.** The final `stdout.log`
    line (what `test.sh` compares) matches the format the grader
    wants: a single letter for MCQ, valid JSON for structured
    answers, only the numeric answer for math. A correct answer in
    the wrong format is a red — it means the prompt didn't convey
    the format, or the agent ignored it.

26. **Reward is plausible for the answer.** Read
    `task/result.json` reward and `task/input/answer.txt`. If reward
    is 0 but the stdout clearly contains the correct answer, the
    grader is broken. If reward is 1 but the answer looks wrong,
    the ground truth is wrong or the grader is too permissive.

### E. Infrastructure hygiene

27. **Cost tracking populated.** `model/result.json` `cost_usd` > 0
    for non-trivial runs. A value of exactly 0 after a 50+ line
    trajectory means the cost logger isn't reading the right field
    (happens for LiteLLM Responses API path in v1.83.3). Note in
    the known-broken manifest; doesn't fail the run on its own.

28. **Timeout was not hit.** `agent/result.json` `exit_code` is not
    124 (GNU timeout code). A timeout-killed run is red — raise
    `EVAL_TIMEOUT` or investigate why the agent looped.

29. **Version axis recorded.** `task/version.json` and
    `agent/version.json` exist and have the pinned default values
    matching the benchmark and agent's
    `EVAL_*_VERSION_DEFAULT` ENVs. If either is empty, RULES.md
    principle 9 is not being honored for that image.

30. **No secrets in logs.** Neither `agent/stdout.log` nor
    `agent/stderr.log` contains `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
    or any token-shaped string the length of a real credential. The
    proxy should be the only code that sees the real key.

### F. Red vs yellow findings

31. **Red** (blocks fixture promotion, goes in `known-broken.md`):
    rule 14 (leaked answer), rule 15 (missing system context), rule
    17 (refusal), rule 19 (wrong task), rule 22 (malformed tool
    calls), rule 25 (wrong format with correct reasoning), rule 30
    (leaked secret). Anything that makes the trace actively
    unreliable as a replay fixture.

32. **Yellow** (fixture still promoted with annotation): rule 18
    (single max_tokens hit), rule 20 (retry storm of 3-5), rule 23
    (minor phantom error later corrected), rule 27 (cost tracking
    missing), rule 28 (near-timeout but not over).

33. **When in doubt, annotate.** If you can't decide red vs yellow,
    record the finding with a yellow severity in the run's
    `provenance.json` entry and move on. A yellow entry is the
    starting point for a next-release-cycle conversation, not a
    blocker now.
