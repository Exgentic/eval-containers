# Output

**Status:** Draft
**Date:** September 2026

## Abstract

Every evaluation, on every deployment surface, leaves one thing behind: its
output. This document fixes what that output is and how it behaves over its
life — where a run writes, what each task directory holds (results, logs,
benchmark artifacts, the run's configuration), when an existing result is
reused, when a dead attempt is retried, when a result is deleted, and when a
run must fail loudly. Compose, single-container, and Kubernetes surfaces all
produce the same output under these rules.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in BCP 14, RFC 2119 and RFC 8174.

A *run* is one launch of one benchmark with one agent and one model over one or
more tasks, named by a *run id*. The *output root* is the directory under which
every run writes. The *run directory* is
`{output root}/{benchmark}/{agent}/{model}/{run-id}/`. A *task directory* is the
directory under the run directory holding one task's `task/`, `agent/`, and
`model/` outputs; it is *complete* when it holds `task/result.json`, whatever
the reward, and *incomplete* otherwise. In the run directory, `{model}` is the
model handle with each `/` replaced by `--` and every other character outside
letters, digits, `.`, `_`, and `-` replaced by `-`. A run's *configuration* is
the benchmark, agent, model, tags, and versions it was launched with. An
*attempt* is one execution of a task. A *forced run* is a run invoked with
`EVAL_FORCE` set. A *disposable run* is a run whose caller has explicitly
declared that its output may be discarded.

## Principles

### Layout

1. **Three directories.** Each task directory MUST hold three separate output directories, `model/`, `agent/`, and `task/`, each owned by exactly one component.

2. **No cross-reads.** No component SHOULD read another component's output directory.

3. **Task result.** `task/result.json` MUST contain at minimum `task_id`, `benchmark`, `reward`, and `passed`.

4. **Named metrics.** Every metric a benchmark reports MUST be a named field in `task/result.json`.

5. **Primary metric.** The metric that determines `passed` MUST be named `reward`.

6. **Graded once.** `task/result.json` MUST be written only by the benchmark's grader.

7. **Not stdout.** A downstream reader MUST take metrics from `task/result.json`, never from stdout.

8. **Agent result.** `agent/result.json` MUST contain `agent`, `started_at`, `ended_at`, and `exit_code`.

9. **Model result.** `model/result.json` MUST contain `model`, `provider`, `total_tokens`, and `cost_usd`.

10. **Trajectory.** The model service MUST write `model/trajectory.jsonl` containing every LLM request and response, one JSON object per line.

11. **Task directory.** Every task MUST write its results in `{output root}/{benchmark}/{agent}/{model}/{run-id}/{task-id}/`.

### Contents

12. **Nothing loose.** Every file in a task directory MUST live under `model/`, `agent/`, or `task/`.

13. **Logs are output.** Every component's log MUST be written into that component's output directory.

14. **Benchmark artifacts.** Every artifact a benchmark collects from the agent's work MUST be written inside the task directory.

15. **Framework files.** A benchmark MUST NOT modify a file written by the framework launcher.

16. **Recorded configuration.** Every run directory MUST hold a file recording the run's configuration.

### Root and run

17. **One root.** Every run MUST write beneath one output root.

18. **Root selector.** The output root MUST be the value of `EVAL_OUTPUT_DIR`, or `output/` in the invoking working directory when it is unset.

19. **Outlives the run.** The output root of a run that is not disposable MUST outlive every container, pod, and job of the run.

20. **Named run.** Every run MUST be named by a run id.

21. **Run id selector.** The run id MUST be the value of `EVAL_RUN_ID`, or a freshly generated unique value when it is unset.

22. **Same configuration.** A run resumed under an existing run id MUST fail before starting when its configuration differs from the recorded one.

23. **Confined writes.** A run MUST NOT write outside its own run directory.

### Reuse, retry, force

24. **Never overwrite.** A run MUST NOT modify a complete task directory.

25. **Reuse.** A task whose task directory is complete MUST be skipped.

26. **Visible reuse.** A skipped task MUST be reported with the path of its existing result.

27. **Retry.** A task whose task directory is incomplete MUST have that directory emptied of the previous attempt before it is attempted again.

28. **Force.** A forced run MUST have every task directory it runs emptied of any previous attempt, complete or not, before the attempt starts.

29. **One attempt.** A task directory MUST hold the files of exactly one attempt.

30. **Bounded deletion.** A run MUST NOT delete any path outside its own run directory.

31. **Single writer.** A task MUST fail before starting when another attempt is in progress in the same task directory.

### Failure

32. **Keep going.** A run MUST continue past a task whose attempt ended incomplete.

33. **Loud failure.** A run MUST exit non-zero when any of its tasks ended incomplete.

34. **Named failure.** A run MUST report the path of every task directory that ended incomplete.

35. **Incomplete is not a score.** An aggregation over an output root MUST count an incomplete task directory as incomplete, never as a reward.

## References

- [Process](../RULES.md) — principle 9 (an output-format change is a major bump), principle 12 (the `EVAL_` namespace).
- [Compose](../compose/RULES.md) — rules 14–18, the deprecated former home of rules 1–11.
- [Benchmarks](../benchmarks/RULES.md) — rule 24 (byte-equivalent `task/result.json` across the three surfaces).
- [CLI](../src/RULES.md) — principle 12 (`--output-dir`, `--run-id`, `--force`) and principle 13 (`report`).
- [Agents](../agents/RULES.md) — rules 3 and 8 (the agent answers on stdout and never touches `/output/task/`).
- [Models](../models/RULES.md) — rules 7 and 11 (only the agent's model service writes `model/`).
- [tests/run/replay/RULES.md](../../tests/run/replay/RULES.md) — the OTLP `traces.jsonl` fixtures derived from the trajectory.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-09-06 | Initial version (#467). Rules 1–10 carried from [compose/RULES.md](../compose/RULES.md) 14–17, split into atomic requirements with paths relative to the task directory. Rule 11 replaces compose 18: the task directory is `{output root}/{benchmark}/{agent}/{model}/{run-id}/{task-id}/`, the layout the cluster launchers already mint (#428) with their model encoding, where the old `output/{benchmark}/{task-id}/` let two agents or models on one task overwrite each other (#136). Rules 12–16 fix what a directory holds: no loose files (today `traces.jsonl` and a second `result.json` sit at the volume root), every component's log in its own directory, benchmark artifacts inside the task directory, framework-written files untouched by benchmarks, and the run's configuration recorded. Rules 17–23 fix root and run: one output root from `EVAL_OUTPUT_DIR` that outlives the run unless the caller declared it disposable (the chart's `ephemeral` escape hatch); a run id from `EVAL_RUN_ID` or freshly generated, so an unnamed rerun never collides and a named one resumes; a resume with a different configuration is refused; writes confined to the run directory. Rules 24–31 fix reuse: a complete task directory is never modified and its task is skipped, visibly; an incomplete one is emptied of the dead attempt and retried; a forced run empties every task directory it runs; one attempt per directory; deletion bounded to the run directory; a second concurrent attempt fails (the output half of #399). Rules 32–35 fix failure: a run keeps going past an incomplete task, exits non-zero, names each incomplete directory, and aggregation never scores one. Resume, lock, recorded-configuration, and one-attempt semantics mirror Harbor's job resume and Inspect's eval-set log directory. |
