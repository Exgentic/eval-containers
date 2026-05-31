# Trajectory Health — signal catalog and reference

Reference material for the `audit-trajectory` skill. This is the detailed
two-half signal catalog, classification rules, collection mechanism, and
layered-checking model that back the ten-question walk in `SKILL.md`. Read it
before walking the questions so you know what the mechanical layer already
covers.

## The inspection unit

For each benchmark-agent combination, one trajectory is inspected:

- **Input:** one `tests/replay/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`
  file, or a live inspector run under `/output/<bench>/<task>/inspector/`. A
  trajectory is an ordered sequence of LiteLLM `StandardLoggingPayload` rows,
  one per LLM call the agent made.
- **Context:** the benchmark name, task ID, expected task shape from the
  benchmark's `eval.benchmark.*` labels, the agent name.
- **Output:** two verdicts (task-half and run-half), each
  `green` | `yellow` | `red`, with matching signals listed. The overall fixture
  verdict is the worse of the two halves.

## Task-half signal catalog

The task half asks: *was the prompt the agent saw well-formed?* Input: the first
non-empty user message in the trajectory.

### Green signals (all must be present for a `green` verdict)

- **Task content present.** At least one `user` role message contains a task
  instruction; the message is non-empty and non-whitespace.
- **Substantive length.** Task text ≥ 50 characters. Shorter almost certainly
  means a template did not render.
- **No unresolved placeholders.** None of: `{TODO}`, `{{placeholder}}`, `%s`,
  `{EVAL_BENCHMARK}`, `${TASK_ID}`, `<INSERT_TASK>`, `FIXME`.
- **Expected format specified.** The prompt mentions what the agent should
  output (`print the answer`, `write to /output`, `return JSON`,
  `final answer:`).
- **Task ID resolved.** The prompt does not contain the literal `$EVAL_TASK_ID`,
  `${EVAL_TASK_ID}`, or `/tasks/$EVAL_TASK_ID`.

### Red signals (any one triggers a `red` verdict)

- **Empty task.** User message is empty, whitespace-only, or shorter than 20
  characters.
- **Unresolved env var.** Task contains literal `$EVAL_BENCHMARK`,
  `${EVAL_BENCHMARK}`, `$EVAL_TASK_ID`, `${EVAL_TASK_ID}`, `$TASK`, `${TASK}`.
- **Fetch failure strings.** Task contains `404 Not Found`, `403 Forbidden`,
  `connection refused`, `TLS handshake`, `dns resolution failed`,
  `certificate verify failed`.
- **File missing strings.** Task contains `no such file`, `permission denied`,
  `cannot open`, `not a directory`, `unable to read`.
- **Template leakage.** Task contains a literal `{NAME}`, `{DATASET}`,
  `{SPLIT}`, `{QUESTION_FIELD}`, `{ANSWER_FIELD}`, `{TASK_PROMPT}` — the
  placeholder names in the benchmark template.
- **Dataset gate.** Task contains `HF_TOKEN required`, `authentication required`,
  `401 Unauthorized`, `access denied`.
- **Binary / encoding garbage.** Characters outside printable UTF-8 beyond what
  the dataset would normally have.
- **Wrong benchmark.** Task content does not match the benchmark name. Hard to
  automate — this is question 1 of the walk.

### Yellow signals (warn but do not fail)

- **Very long task** (> 10k chars) — might be legitimate, might be a template
  runaway that concatenated the whole dataset.
- **No clear instruction verb.** Lacks any of `solve`, `write`, `compute`,
  `translate`, `answer`, `find`, `explain`, `return`, `print`.
- **No expected-format hint.** Does not tell the agent where to put its answer.
- **Suspiciously short** (20 ≤ len < 50 chars).
- **Attached files not referenced.** The benchmark's `/tasks/<id>/` contains
  image or document files but the prompt does not mention any file path.

## Run-half signal catalog

The run half asks: *was the actual conversation sane?* Input: every row. A
LiteLLM row has `status`, `total_tokens`, `response_cost`, `error_str`,
`error_information`, and the full `response`.

### Red signals (any one triggers a `red` run verdict)

- **API error.** Any row has `status != "success"`, OR non-empty `error_str`,
  OR non-empty `error_information.error_message`.
- **All-empty assistant responses.** Every assistant turn has empty `content`
  and no `tool_calls`.
- **Context overflow.** Any row's error contains `context_length_exceeded`,
  `context window`, or `maximum context length`.
- **Auth failure.** Any row's error mentions `401`, `403`, `authentication`,
  `invalid api key`, or `permission denied` at the API layer.

### Yellow signals (warn but do not fail)

- **Cost runaway.** Sum of `response_cost` > $5 for a single task.
- **Token runaway.** Sum of `total_tokens` > 200k for a single task.
- **Retry storm.** The same prompt is sent 5+ times in a row without material
  change.
- **High turn count.** More than 100 rows for a single task.
- **Empty final response.** The last assistant turn has empty content and no
  tool calls.
- **Tool error loop.** The same tool error string appears in 3+ consecutive
  assistant turns.

### Green signals (all must be present for a `green` run verdict)

- **No API errors.** Every row `status == "success"`, empty `error_str`, empty
  `error_information.error_message`.
- **Cost under cap.** Sum of `response_cost` ≤ $5.
- **Tokens under cap.** Sum of `total_tokens` ≤ 200k.
- **Turn count reasonable.** Fewer than 100 LLM calls.
- **Final response non-empty.** The last assistant turn contains content or at
  least one tool call.

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

A fixture with all greens on both halves is fully healthy. Yellow is worth human
review but not a CI failure. Red is a CI failure.

## Collection mechanism

The `inspector` model (`models/inspector/`) is a tiny Flask app that:

1. Listens on port 4000, serving `/v1/chat/completions`, `/v1/messages`,
   `/v1/responses` (all three API shapes agents use).
2. On the first request, writes the full request body to
   `/output/inspector/first_request.json` inside the mounted output volume, plus
   a summary line to `/output/inspector/summary.txt`.
3. Returns a minimal response that ends the agent's turn and exits cleanly
   (OpenAI: empty-text assistant message with `finish_reason="stop"`; Anthropic:
   a `stop` stop_reason).
4. Does NOT call any upstream provider. Zero API cost.

Usage:

```bash
EVAL_BENCHMARK=aime EVAL_TASK_ID=0 EVAL_AGENT=claude-code EVAL_MODEL=inspector \
  docker compose -f oci://quay.io/eval-containers/evaluate up --abort-on-container-exit
```

Output lands at `output/aime/0/inspector/first_request.json`.

## Layered checking

**Layer 1 — mechanical rules (automated, free, always on).** Every signal in
both catalogs above is a regex, length, or sum check. The sanity trajectory rule
catalog (`tests/sanity/RULES.md:6`) walks trajectory records and
applies two rule sets — one for the task half (first user message), one for the
run half (every row). Runs in milliseconds on every `cargo test`. Catches the
90% of breakage that is mechanical: template leaks, unresolved env vars, fetch
failures, API errors, cost runaways, empty responses.

**Layer 2 — procedural audit (manual, on demand).** The ten-question walk in
`SKILL.md`. Some judgments need reading and thinking, not regex: is the task
clear enough for a competent human? Does it match the benchmark's domain? Are
attached files referenced where the benchmark needs them? Does the expected
format make sense? A reviewer can be a person, an AI assistant, or a script
implementing the mechanizable parts; the output format is fixed.

**Layer 3 — delta monitoring (future).** Snapshot layer-1 verdicts per benchmark
after a known-good run; alert when a benchmark transitions green → yellow/red.

**Layer 4 — provenance check (future).** Verify the task content hash matches
what is expected from the pinned `eval.benchmark.data_revision`.

## Output format

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

## References

- `benchmarks/RULES.md` — what a benchmark must produce.
- `tests/replay/RULES.md` — fixture lifecycle, the broken
  manifest, provenance.
- `tests/live/RULES.md:13-33` — the live trace-inspection
  checklist this audit mirrors.
- `models/inspector/` — the collection mechanism.
- `doctrine/verification/audit-dockerfile/references/checklist.md` — parallel
  spec for static Dockerfile health.
