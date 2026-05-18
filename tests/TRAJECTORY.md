# Trajectory Health Inspection

**Status:** Draft
**Date:** April 2026

## Abstract

Structural tests (compose parses, Dockerfile builds) tell you the
benchmark is **well-formed**. They do not tell you the benchmark is
**legitimate**, and they do not tell you the agent's actual run was
**sane**. A benchmark can build perfectly, start the agent cleanly,
hand it an empty task, and produce a valid-looking `result.json`
that scores zero. An agent can be handed a perfect task, burn 200k
tokens in a retry storm, hit an API error, and produce garbage — and
still report a plausible score.

Trajectory health inspection closes both gaps. It has **two halves**:

1. **Task half** — was the prompt the agent saw well-formed? Captured
   from the first user message in the trajectory.
2. **Run half** — was the actual conversation sane? Read from every
   row in the trajectory: API status, tokens, cost, retries, tool
   errors, empty responses.

Both halves use the same layered model (mechanical rules + procedural
audit) and produce findings in the same format.

This document defines what we look for, how we classify signals, and
how the inspection becomes progressively more automated.

## The inspection unit

For each benchmark-agent combination, one trajectory is inspected:

- **Input:** one `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`
  file, or a live inspector run under `/output/<bench>/<task>/inspector/`.
  A trajectory is an ordered sequence of LiteLLM `StandardLoggingPayload`
  rows, one per LLM call the agent made.
- **Context:** the benchmark name, task ID, expected task shape from
  the benchmark's `eval.benchmark.*` labels, the agent name.
- **Output:** two verdicts (task-half and run-half), each
  `green` | `yellow` | `red`, with the matching signals listed.
  The overall fixture verdict is the worse of the two halves.

## Task-half signal catalog

The task half asks: *was the prompt the agent saw well-formed?*
Inputs: the first non-empty user message in the trajectory.

### Green signals (all must be present for a `green` verdict)

- **Task content present.** At least one `user` role message contains
  a task instruction. The message is non-empty and non-whitespace.
- **Substantive length.** Task text ≥ 50 characters. Real benchmarks
  always have instructions longer than this; shorter almost certainly
  means a template didn't render.
- **No unresolved placeholders.** None of: `{TODO}`, `{{placeholder}}`,
  `%s`, `{EVAL_BENCHMARK}`, `${TASK_ID}`, `<INSERT_TASK>`, `FIXME`.
  These indicate the entrypoint didn't substitute variables.
- **Expected format specified.** The prompt mentions what the agent
  should output (`print the answer`, `write to /output`, `return
  JSON`, `final answer:`). Missing this is usually fine for open-ended
  tasks but worth flagging.
- **Task ID resolved.** The prompt does not contain the literal string
  `$EVAL_TASK_ID` or `${EVAL_TASK_ID}` or `/tasks/$EVAL_TASK_ID`.
  Presence means variable substitution failed.

### Red signals (any one triggers a `red` verdict)

- **Empty task.** User message is empty, whitespace-only, or shorter
  than 20 characters.
- **Unresolved env var.** Task contains literal `$EVAL_BENCHMARK`,
  `${EVAL_BENCHMARK}`, `$EVAL_TASK_ID`, `${EVAL_TASK_ID}`, `$TASK`,
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

## Run-half signal catalog

The run half asks: *was the actual conversation sane?* Inputs: every
row in the trajectory, not just the first. A LiteLLM row has
`status`, `total_tokens`, `response_cost`, `error_str`,
`error_information`, and the full `response` (assistant message +
tool calls). The rules walk the sequence and accumulate findings.

### Red signals (any one triggers a `red` run verdict)

- **API error.** Any row has `status != "success"`, OR non-empty
  `error_str`, OR non-empty `error_information.error_message`. The
  conversation hit a transport-level failure the agent couldn't see.
- **All-empty assistant responses.** Every assistant turn has empty
  `content` and no `tool_calls`. The agent produced zero output for
  the whole run — something upstream refused or stalled.
- **Context overflow.** Any row has an error class or message
  containing `context_length_exceeded`, `context window`, or
  `maximum context length`. The agent blew past the model's limit.
- **Auth failure.** Any row's error mentions `401`, `403`,
  `authentication`, `invalid api key`, or `permission denied` at
  the API layer. Proxy config is broken.

### Yellow signals (warn but don't fail)

- **Cost runaway.** Sum of `response_cost` across all rows > $5 for
  a single task. Real benchmarks almost never cost that much; usually
  means the agent looped.
- **Token runaway.** Sum of `total_tokens` across all rows > 200k for
  a single task. Same story as cost runaway.
- **Retry storm.** The same prompt is sent 5+ times in a row without
  material change. Indicates the agent is stuck on a failing tool or
  API.
- **High turn count.** More than 100 rows for a single task. Might
  be legitimate for a long-running coding task, but usually a sign
  the agent is thrashing.
- **Empty final response.** The last assistant turn has empty content
  AND no tool calls. The run ended with nothing being said.
- **Tool error loop.** The same tool error string appears in 3+
  consecutive assistant turns. The agent isn't learning from the
  error.

### Green signals (all must be present for a `green` run verdict)

- **No API errors.** Every row has `status == "success"`, empty
  `error_str`, empty `error_information.error_message`.
- **Cost under cap.** Sum of `response_cost` ≤ $5.
- **Tokens under cap.** Sum of `total_tokens` ≤ 200k.
- **Turn count reasonable.** Fewer than 100 LLM calls.
- **Final response non-empty.** The last assistant turn contains
  either content or at least one tool call.

## Classification rules

```
task_verdict = red   if any red task signal
             = yellow if any yellow task signal
             = green  otherwise

run_verdict  = red   if any red run signal
             = yellow if any yellow run signal
             = green  otherwise

fixture_verdict = worst(task_verdict, run_verdict)
```

A fixture with all greens on both halves is fully healthy. Yellow is
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
EVAL_BENCHMARK=aime EVAL_TASK_ID=0 EVAL_AGENT=claude-code EVAL_MODEL=inspector \
  docker compose -f oci://quay.io/eval-containers/evaluate up --abort-on-container-exit
```

Output lands at `output/aime/0/inspector/first_request.json`.

## Layered checking

Health inspection is layered. Each layer catches what the layer below
misses, at increasing cost and judgment.

**Layer 1 — mechanical rules (automated, free, always on).**
Every signal in both catalogs above is a regex, length, or sum
check. The test file `tests/task_inspection.rs` walks trajectory
records and applies two rule catalogs — one for the task half (first
user message), one for the run half (every row). Runs in
milliseconds. Runs on every `cargo test`. Catches the 90% of real
breakage that's mechanical: template leaks, unresolved env vars,
fetch failures, API errors, cost runaways, empty responses. Zero
dependencies.

**Layer 2 — procedural audit (manual, on demand).**
Some judgments need reading and thinking, not regex:
- "Is the task instruction clear enough for a competent human to
  attempt it?"
- "Does the task match the benchmark's stated domain? (the prompt
  says 'translate' but the benchmark is a coding benchmark)"
- "Are attached files referenced where the benchmark needs them?"
- "Does the expected output format make sense?"

The "Audit procedure" section below is a checklist a reviewer walks
through manually. A reviewer can be a person reading the docs, an
AI assistant executing the checklist, or a script implementing
mechanizable parts. The procedure is the same regardless of who runs
it; the output format is fixed so findings are comparable.

Run procedural audits on demand — before releases, quarterly health
checks, when mechanical rules flag a yellow, when a new benchmark
batch lands.

**Layer 3 — delta monitoring (future, catches regressions).**
Snapshot the layer 1 verdicts per benchmark after a known-good run.
Alert when a benchmark transitions from green to yellow/red — that's
a regression, not a legitimate change.

**Layer 4 — provenance check (future, catches supply chain issues).**
Verify the task content hash matches what's expected from the pinned
`eval.benchmark.data_revision`. If upstream silently changed the
dataset under a revision pin, this catches it.

## Audit procedure

Run this when you want a judgment-level review of trajectory health.
Applies to a single fixture or batch.

### Scope

- **Input:** one or more files under `tests/fixtures/*.trajectory.jsonl`,
  or a directory of `inspector` outputs from a live run.
- **Context for each:** the benchmark name, benchmark description from
  the `eval.benchmark.description` label, any special notes in
  `benchmarks/<name>/README.md` if present.

### Steps

1. **Extract the task.** Open the trajectory file, read rows until
   you find the first row with a non-empty user message. Concatenate
   every `role: "user"` message in that row into the task text. Ignore
   system and developer messages — they're framework scaffolding.

2. **Run mechanical rules first.** If `cargo test --test task_inspection
   inspect_every_existing_fixture -- --ignored` flagged anything,
   note those findings. The audit's job is to find what rules missed,
   not to duplicate them.

3. **Read the task end to end (task half).** Ask the five questions
   below about the first user message. Mark each yes / no / n.a. with
   a one-line reason.

   | # | Question |
   |---|---|
   | 1 | Does the task match the benchmark's stated domain? |
   | 2 | Is the instruction clear enough for a competent human? |
   | 3 | Is the expected output format obvious from the prompt? |
   | 4 | If the benchmark needs attached files (images, docs, repos), does the prompt reference them? |
   | 5 | Any subtle signs of a broken environment you'd only catch by reading (dangling references, contradictory instructions, wrong task ID in the prompt)? |

4. **Read the run end to end (run half).** Scroll through the
   assistant turns and tool calls. Ask the five questions below
   about the conversation as a whole.

   | # | Question |
   |---|---|
   | 6 | Did the agent's first substantive response actually attempt the task, or refuse / stall / ask for clarification? |
   | 7 | Are there repeated tool errors, retry storms, or identical messages sent multiple times in a row? |
   | 8 | Does the assistant's final answer match the recorded score? (score=1 → is the answer actually correct? score=0 → do you see why it failed?) |
   | 9 | Any API errors, context overflows, or auth/network failures mid-run? |
   | 10 | Is the cost / token count reasonable for this task, or did the agent burn resources in a loop? |

5. **Classify each half.**
   - ✓ **healthy** — all yes or n.a. in that half.
   - ⚠ **needs attention** — task-half Q2/Q3/Q4 no, or run-half Q7/Q10 no (fixable).
   - ✗ **broken** — task-half Q1/Q5 no, or run-half Q6/Q8/Q9 no (the benchmark or run is wrong).

   **Overall fixture verdict is the worse of the two halves.**

### Output format

One markdown report, one entry per fixture:

```
## aime-0-claude-code
- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ math problem matches AIME
  - Q2 (clear): ✓ "solve... print only the answer as a single integer"
  - Q3 (format): ✓ explicit single-integer instruction
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ first turn jumps into the problem
  - Q7 (retry/loops): ✓ no repetition
  - Q8 (score credible): ✓ final answer "204" matches recorded score=1
  - Q9 (API errors): ✓ all rows status=success
  - Q10 (cost sane): ✓ $0.03, 13k tokens
- Verdict: healthy
```

Followed by a summary count and 3 suggested fixes (if any).

### When to run

- Before cutting a release (whole fleet)
- When `task_inspection` flags a yellow that needs judgment
- When a new benchmark batch lands in the repo
- Quarterly, as a health check

### Who runs it

Anyone. The procedure is toolchain-agnostic. A human walks through
with `less` and a notepad. An AI assistant reads this doc and executes
the checklist. A script implements the mechanizable parts (the rule
engine already does layer 1). All three produce the same report shape.

## References

- [Benchmarks RULES](RULES.md) — what a benchmark must produce
- [Replay fixtures](../tests/fixtures/) — existing trajectory format
- [Inspector model](../models/inspector/) — the collection mechanism
