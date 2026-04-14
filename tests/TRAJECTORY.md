# Trajectory Health Inspection

**Status:** Draft
**Date:** April 2026

## Abstract

Structural tests (compose parses, Dockerfile builds) tell you the
benchmark is **well-formed**. They do not tell you the benchmark is
**legitimate**. A benchmark can build perfectly, start the agent
cleanly, hand it an empty task, and produce a valid-looking
`result.json` that scores zero — all while being completely broken.

Trajectory health inspection closes that gap. For every benchmark,
we capture the **first request** the agent sends to the LLM proxy
and inspect it against a checklist of green, yellow, and red signals.

This document defines what we look for, how we classify signals, and
how the inspection becomes progressively more automated.

## The inspection unit

For each benchmark, one record is inspected:

- **Input:** the first OpenAI / Anthropic-style request body the agent
  sends to the model proxy, captured via the `inspector` model.
- **Context:** the benchmark name, task ID, expected task shape from
  the benchmark's `dock.benchmark.*` labels.
- **Output:** a health verdict (`green` | `yellow` | `red`) with the
  matching signals listed.

## Signal catalog

### Green signals (all must be present for a `green` verdict)

- **Task content present.** At least one `user` role message contains
  a task instruction. The message is non-empty and non-whitespace.
- **Substantive length.** Task text ≥ 50 characters. Real benchmarks
  always have instructions longer than this; shorter almost certainly
  means a template didn't render.
- **No unresolved placeholders.** None of: `{TODO}`, `{{placeholder}}`,
  `%s`, `{DOCK_BENCHMARK}`, `${TASK_ID}`, `<INSERT_TASK>`, `FIXME`.
  These indicate the entrypoint didn't substitute variables.
- **Expected format specified.** The prompt mentions what the agent
  should output (`print the answer`, `write to /output`, `return
  JSON`, `final answer:`). Missing this is usually fine for open-ended
  tasks but worth flagging.
- **Task ID resolved.** The prompt does not contain the literal string
  `$DOCK_TASK_ID` or `${DOCK_TASK_ID}` or `/tasks/$DOCK_TASK_ID`.
  Presence means variable substitution failed.

### Red signals (any one triggers a `red` verdict)

- **Empty task.** User message is empty, whitespace-only, or shorter
  than 20 characters.
- **Unresolved env var.** Task contains literal `$DOCK_BENCHMARK`,
  `${DOCK_BENCHMARK}`, `$DOCK_TASK_ID`, `${DOCK_TASK_ID}`, `$TASK`,
  `${TASK}` — variable substitution failed.
- **Fetch failure strings.** Task contains `404 Not Found`,
  `403 Forbidden`, `connection refused`, `TLS handshake`,
  `dns resolution failed`, `certificate verify failed`.
- **File missing strings.** Task contains `no such file`,
  `permission denied`, `cannot open`, `not a directory`,
  `unable to read`.
- **Template leakage.** Task contains a literal `{NAME}`, `{DATASET}`,
  `{SPLIT}`, `{QUESTION_FIELD}`, `{ANSWER_FIELD}`, `{TASK_PROMPT}` —
  these are the placeholder names in `benchmarks/TEMPLATE.md` and
  indicate the author forgot to fill in the template.
- **Dataset gate.** Task contains `HF_TOKEN required`,
  `authentication required`, `401 Unauthorized`, `access denied`.
- **Binary / encoding garbage.** Task contains characters outside
  printable UTF-8 beyond what the dataset would normally have.
- **Wrong benchmark.** Task content does not match the benchmark
  name — e.g. the prompt says "translate to French" but the
  benchmark is `humaneval`. Hard to automate; see "Human review" below.

### Yellow signals (trigger a warning but not a failure)

- **Very long task** (> 10k chars) — might be legitimate, might be a
  template runaway that concatenated the whole dataset.
- **No clear instruction verb.** Task lacks any of: `solve`, `write`,
  `compute`, `translate`, `answer`, `find`, `explain`, `return`,
  `print`. May be fine for freeform tasks.
- **No expected-format hint.** Task doesn't tell the agent where to
  put its answer. Usually fine, sometimes a mistake.
- **Suspiciously short** (20 ≤ len < 50 chars) — borderline, worth
  a human glance.
- **Attached files not referenced.** The benchmark's `/tasks/<id>/`
  contains image or document files but the prompt doesn't mention
  any file path like `/app/image.png`.

## Classification rules

```
if any red signal:        verdict = red
elif any yellow signal:   verdict = yellow
else:                     verdict = green
```

A run with all greens and zero yellows is fully healthy. Yellow is
worth human review but not a CI failure. Red is a CI failure.

## Collection mechanism

The `inspector` model (`models/inspector/`) is a tiny Flask app that:

1. Listens on port 4000, serves `/v1/chat/completions`, `/v1/messages`,
   `/v1/responses` (all three API shapes agents use).
2. On the first request, writes the full request body to
   `/output/inspector/first_request.json` inside the mounted output
   volume. Also writes a summary line to `/output/inspector/summary.txt`.
3. Returns a minimal response that ends the agent's turn and exits
   cleanly. For OpenAI, return an empty-text assistant message with
   `finish_reason="stop"`. For Anthropic, return a `stop` stop_reason.
4. Does NOT call any upstream provider. Zero API cost.

Usage:

```bash
DOCK_BENCHMARK=aime DOCK_TASK_ID=0 DOCK_AGENT=claude-code DOCK_MODEL=inspector \
  docker compose -f oci://quay.io/dock-eval/evaluate up --abort-on-container-exit
```

Output lands at `output/aime/0/inspector/first_request.json`.

## Automation roadmap

**Phase A — rule-based (automatable, ship first):**
Every signal in the catalog above is a regex or length check. A single
test file (`tests/task_inspection.rs`) walks benchmarks, runs the
inspector, reads the first request, applies the rules, reports
verdicts. Zero dependencies beyond what's already in the repo.

**Phase B — LLM-as-judge (opt-in, costs API dollars):**
For borderline yellow cases and "wrong benchmark" detection, feed the
trajectory snippet + the benchmark name to a cheap model and ask
"does this task match the benchmark?". Only runs on yellow verdicts
by default, so the dollar cost is bounded.

**Phase C — delta monitoring (catches regressions):**
Snapshot the green verdicts per benchmark after a known-good run.
Compare new runs against the snapshot. Alert on any benchmark that
transitions from green to yellow/red — that's a regression, not a
legitimate change.

**Phase D — provenance check (catches supply chain issues):**
Verify the task content hash matches what's expected from the
pinned `dock.benchmark.data_revision`. If upstream silently changed
the dataset under a revision pin, this catches it.

Phases A through D layer additively. Ship A now; add the rest as need
arises.

## Human review

Some judgments are hard to automate:
- "Is the task instruction clear?"
- "Would a competent human understand what to do?"
- "Is the expected answer format reasonable?"
- "Does the task match the benchmark's stated domain?"

Until LLM-as-judge handles these reliably, yellow verdicts ship to a
reviewer queue. The reviewer either upgrades to green (tune the rules)
or downgrades to red (fix the benchmark).

## References

- [Benchmarks RULES](RULES.md) — what a benchmark must produce
- [Replay fixtures](../tests/fixtures/) — existing trajectory format
- [Inspector model](../models/inspector/) — the collection mechanism
