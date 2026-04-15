# Trajectory Audit — 2026-04-15

Commit: `90244c2`
Walked by: procedural audit per tests/TRAJECTORY.md
Fixtures: 23

Mechanical-rule baseline used as input: task_inspection.rs reports 0 task-half
findings across all 23 fixtures and 7 run-half yellows: token_runaway on
gaia/gdpval/mrcr and retry_storm on gdpval/hle/mrcr/simpleqa.

## Per-fixture verdicts

### aider-polyglot-0-aider

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ C++ exercise "all_your_base" (base conversion) matches Aider Polyglot domain
  - Q2 (clear): ✓ explicit edit-these-files instructions plus the Exercism README
  - Q3 (format): ✓ aider SEARCH/REPLACE block format is specified
  - Q4 (attachments): ✓ file contents inlined in user message with "added these files to the chat"
  - Q5 (subtle breakage): ✓ task well-formed; earlier primer turns ("get_factorial", "hello()") are aider's expected few-shot priming
- Run half:
  - Q6 (attempted task): ✓ assistant produces SEARCH/REPLACE blocks targeting all_your_base.cpp/.h immediately
  - Q7 (retry/loops): ⚠ 7 of 11 rows fail with `max_tokens` / output limit; subsequent fix-error-below rounds repeat similar edits
  - Q8 (score credible): n.a. — final blocks still failing to exact-match; run ended mid-edit so score=0 plausible
  - Q9 (API errors): ✗ 7 rows carry `litellm.BadRequestError … max_tokens or model output limit was reached`
  - Q10 (cost sane): ✓ 18.5k tokens, $0.05
- Verdict: **broken** (Q9 API errors dominate the run)

### aime-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ AIME-style quadratic problem matches benchmark
  - Q2 (clear): ✓ clear single-answer math problem
  - Q3 (format): ✓ "Print only the answer as a single integer"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ first substantive turn tries to set up P(x)=2x²+bx+c / Q(x)=-2x²+dx+e
  - Q7 (retry/loops): ⚠ 2 rows degenerate to single-word "Hey" after `finish_reason=length`
  - Q8 (score credible): ✗ final answer "43"; this AIME problem's accepted answer is 116, so a recorded score=1 would be wrong
  - Q9 (API errors): ⚠ 2 `max_tokens` truncations but run recovered
  - Q10 (cost sane): ✓ 54k tokens, $0.14
- Verdict: **broken** (Q8: final answer is incorrect for a verifiable math benchmark)

### appworld-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ Splitwise/Venmo simulated-app task matches AppWorld (457-API sandbox)
  - Q2 (clear): ✓ describes the intent clearly for a human
  - Q3 (format): ⚠ no explicit output shape — implied "perform API calls" but never stated
  - Q4 (attachments): ⚠ references "attached Venmo receipt" but no receipt file path is in the prompt
  - Q5 (subtle breakage): ⚠ prompt assumes the agent already has Splitwise / Venmo API handles but those are not referenced
- Run half:
  - Q6 (attempted task): ✗ assistant refuses with "I can't do that on your behalf. This would require interacting with third-party financial accounts…"
  - Q7 (retry/loops): ⚠ 4 `max_tokens` failures but no thrash
  - Q8 (score credible): ✗ agent refused outright — any recorded non-zero score would be spurious
  - Q9 (API errors): ⚠ 2 `content_filter` finishes plus 4 `max_tokens`, but ended cleanly
  - Q10 (cost sane): ✓ 0 tokens recorded, $0
- Verdict: **broken** (Q6: hard refusal + Q4/Q5 task-half deficiencies — agent was never wired into the AppWorld sandbox)

### arc-agi-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ grid-reasoning task matches ARC-AGI-2
  - Q2 (clear): ✓ standard ARC framing with training examples and a test input
  - Q3 (format): ✓ "Print ONLY the output grid as a JSON array of arrays"
  - Q4 (attachments): ✓ grids inlined as text
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ⚠ first content is a refusal ("I'm sorry, but I cannot assist with that request.") — content_filter fired, then recovered
  - Q7 (retry/loops): ⚠ 1 failure row, 1 content_filter, 1 success — short but erratic
  - Q8 (score credible): n.a. — final grid `[[3,7,6],[1,9,9],…]` is plausible but unverifiable by eye
  - Q9 (API errors): ⚠ 1 row `max_tokens` error, 1 content_filter
  - Q10 (cost sane): ✓ 59k tokens, $0.08
- Verdict: **needs attention** (Q6 first turn refused before recovering — spurious safety block)

### bfcl-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ tool-use call for `calculate_triangle_area` matches BFCL
  - Q2 (clear): ✓ function schema + user ask
  - Q3 (format): ✓ "Print ONLY the function call(s) as JSON"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ assistant emits the correct JSON call
  - Q7 (retry/loops): ✓ single row, no retries
  - Q8 (score credible): ✓ `{"name":"calculate_triangle_area","arguments":{"base":10,"height":5,"unit":"units"}}` is the expected answer
  - Q9 (API errors): ✓ status=success
  - Q10 (cost sane): ✓ 13k tokens, $0.03
- Verdict: **healthy**

### browsecomp-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ web-search factual question matches BrowseComp
  - Q2 (clear): ✓ detailed riddle about an African author / probation officer years
  - Q3 (format): ✓ "Print only the answer, nothing else"
  - Q4 (attachments): n.a. (task is research-only, no files needed)
  - Q5 (subtle breakage): ⚠ codex agent has no browsing tool in the trajectory — the task assumes web access that was not provided
- Run half:
  - Q6 (attempted task): ⚠ only 1 of 7 rows produced content; agent guessed "1988 to 1996" without browsing
  - Q7 (retry/loops): ⚠ 6 consecutive `max_tokens` failures
  - Q8 (score credible): n.a. (no expected answer to compare against in this fixture)
  - Q9 (API errors): ✗ 6 `max_tokens` errors out of 7 rows
  - Q10 (cost sane): ✓ 25k tokens, $0.07
- Verdict: **broken** (Q9 dominant API failures + Q5 missing browsing tool)

### gaia-0-goose

- Mechanical rules: ✓ task (0 findings), ⚠ run (token_runaway 311k)
- Task half:
  - Q1 (domain match): ✓ multi-hop factual task matches GAIA
  - Q2 (clear): ✓ concrete question about an arXiv AI-regulation paper
  - Q3 (format): ✓ "Print only the answer, nothing else"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ⚠ goose adds "Generate a short title for the above messages." — framework scaffolding leaking into the prompt
- Run half:
  - Q6 (attempted task): ✓ first content is a brief title ("AI regulation paper"), later the question is answered
  - Q7 (retry/loops): ⚠ 18 of 37 rows failed with `max_tokens`
  - Q8 (score credible): n.a. — "egalitarian" is a plausible word from a three-axis figure but not verifiable here
  - Q9 (API errors): ⚠ 18 `max_tokens` failures mid-run
  - Q10 (cost sane): ✗ 311k tokens (exceeds 200k cap) — mechanical runaway confirmed by reading the trace
- Verdict: **needs attention** (Q10 token runaway confirmed; Q9 a recoverable pattern)

### gdpval-0-claude-code

- Mechanical rules: ✓ task (0 findings), ⚠ run (token_runaway 435k, retry_storm)
- Task half:
  - Q1 (domain match): ✓ audit / spreadsheet variance analysis matches GDPval professional-work domain
  - Q2 (clear): ✓ five numbered sub-tasks (sample-size, variance, sample selection, testing, write-up)
  - Q3 (format): ✓ tabs in the spreadsheet specified by name
  - Q4 (attachments): ✗ references "attached spreadsheet titled 'Population'" but no file path in the prompt — agent has nowhere to read it from
  - Q5 (subtle breakage): ⚠ the workload depends on an artifact that isn't referenced
- Run half:
  - Q6 (attempted task): ✗ first substantive content is "I'm sorry, but I cannot assist with that request." — 13 rows hit `content_filter`
  - Q7 (retry/loops): ✗ 66 rows with 38 `max_tokens` failures + 16 content_filter — mechanical retry_storm corroborated
  - Q8 (score credible): n.a. (open-ended audit report)
  - Q9 (API errors): ✗ 38 transport failures
  - Q10 (cost sane): ✗ 435k tokens, $0.36 — mechanical runaway confirmed
- Verdict: **broken** (Q4 task missing spreadsheet path, Q6 refusal loop, Q9 pervasive API errors)

### gpqa-diamond-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ graduate-level astro question matches GPQA Diamond
  - Q2 (clear): ✓ exoplanet density MCQ
  - Q3 (format): ✓ "Print only the letter (A, B, C, or D)"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ first content is "D" (which maps to option "c" — same composition, 5× mass)
  - Q7 (retry/loops): ✓ short run, no loops
  - Q8 (score credible): ✗ expected answer is "C" (option b: density 5.5 g/cm³, which is higher than 5× Earth composition); agent picked "D" — wrong
  - Q9 (API errors): ⚠ 1 `max_tokens` failure row
  - Q10 (cost sane): ✓ 26k tokens, $0.07
- Verdict: **broken** (Q8 wrong answer on a verifiable MCQ)

### healthbench-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ postpartum depression planning matches HealthBench
  - Q2 (clear): ✓ specific patient context and explicit asks (plan, therapy, steps)
  - Q3 (format): ⚠ no explicit output format — a 3-month plan is implied
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ first substantive row is a structured 3-month PPD plan with safety guidance
  - Q7 (retry/loops): ⚠ 16 of 36 rows `max_tokens` but run kept recovering
  - Q8 (score credible): n.a. — rubric-scored; final row is a grader JSON with explanation
  - Q9 (API errors): ⚠ 16 `max_tokens` truncations
  - Q10 (cost sane): ✓ 71k tokens, $0.13
- Verdict: **needs attention** (high failure rate but recovered)

### hle-0-claude-code

- Mechanical rules: ✓ task (0 findings), ⚠ run (retry_storm)
- Task half:
  - Q1 (domain match): ✓ chess-mate question matches HLE's hard-exam domain
  - Q2 (clear): ✓ "Black to move, mate in 2 for black, don't move black queens"
  - Q3 (format): ✓ "Print only the final answer, nothing else. Be as concise as possible"
  - Q4 (attachments): ✓ points to `/app/image.txt` for image data
  - Q5 (subtle breakage): ⚠ `/app/image.txt` is referenced but the agent never opens it in any visible tool call
- Run half:
  - Q6 (attempted task): ✗ first 4 substantive rows are `content_filter` refusals ("I'm sorry, but I cannot assist with that request.") before finally emitting "Qh2#"
  - Q7 (retry/loops): ⚠ 18 max_tokens failures + 4 content_filter in 31 rows — retry_storm confirmed
  - Q8 (score credible): n.a. — "Qh2#" is only half a mate-in-2 and unverifiable here
  - Q9 (API errors): ⚠ 18 `max_tokens` errors mid-run
  - Q10 (cost sane): ⚠ 191k tokens, $0.06 — just under the 200k cap but in the same shape as runaway fixtures
- Verdict: **broken** (Q6 initial refusals; retry_storm corroborated)

### humaneval-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ `has_close_elements` is canonical HumanEval/0
  - Q2 (clear): ✓ signature + docstring + examples
  - Q3 (format): ✓ "Print ONLY the function body"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ emits correct sorted-diff implementation
  - Q7 (retry/loops): ✓ 2 rows, one failure recovered to completion
  - Q8 (score credible): ✓ code is correct (returns True iff any sorted neighbor pair is within threshold)
  - Q9 (API errors): ⚠ 1 `max_tokens` failure then success
  - Q10 (cost sane): ✓ 20k tokens, $0.05
- Verdict: **needs attention** (transient API error; result correct)

### ifeval-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ constrained writing task matches IFEval
  - Q2 (clear): ✓ 300+ word summary, no commas, ≥3 highlighted sections
  - Q3 (format): ✓ markdown `*highlighted*` format specified
  - Q4 (attachments): ⚠ asks agent to summarize a Wikipedia URL; assumes web access the trajectory doesn't show
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ⚠ 2 rows refused with "I'm sorry…" before agent produced a plan+overview
  - Q7 (retry/loops): ⚠ 4 `max_tokens` failures out of 7
  - Q8 (score credible): n.a. (verifiable-constraint task; cannot count commas without the full output)
  - Q9 (API errors): ⚠ 4 `max_tokens` errors plus 2 `content_filter`
  - Q10 (cost sane): ✓ 21k tokens, $0.01
- Verdict: **needs attention** (content_filter + fetch assumption)

### kumo-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ pick-next-action from diagnostic observations matches Kumo
  - Q2 (clear): ✓ observation dict + available-actions list
  - Q3 (format): ✓ "Print ONLY the action, nothing else"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ⚠ prompt already contains "Reaction with iodine solution": "iodoform_test_positive" as an OBSERVATION, and the agent's answer re-emits "Reaction with iodine solution" — possible contradiction between test vs. action framing
- Run half:
  - Q6 (attempted task): ✓ outputs a valid action from the list
  - Q7 (retry/loops): ✓ 2 rows, clean
  - Q8 (score credible): n.a. — cannot verify the correct next diagnostic action from the fixture alone
  - Q9 (API errors): ⚠ 1 `max_tokens` failure then success
  - Q10 (cost sane): ✓ 13k tokens, $0.03
- Verdict: **needs attention** (Q5 ambiguity in task framing, minor)

### livecodebench-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ abc-swap competitive-programming problem matches LiveCodeBench
  - Q2 (clear): ✓ full problem statement + samples
  - Q3 (format): ✓ "Print ONLY the complete source code"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ emits a Python solution that reads t cases and outputs YES/NO
  - Q7 (retry/loops): ✓ clean
  - Q8 (score credible): ✓ straightforward "one swap away from abc" check — correct logic
  - Q9 (API errors): ⚠ 1 `max_tokens` failure then success
  - Q10 (cost sane): ✓ 14k tokens, $0.04
- Verdict: **needs attention** (transient API error, result correct)

### math-500-0-aider

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ convert (0,3) to polar matches MATH-500
  - Q2 (clear): ✓ standard rect→polar request
  - Q3 (format): ✓ "Print only the final answer in its simplest form"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓ — the earlier `get_factorial()` / `hello()` turns are aider's standard few-shot priming, not leakage
- Run half:
  - Q6 (attempted task): ✓ single-call run, answer emitted
  - Q7 (retry/loops): ✓ 1 row
  - Q8 (score credible): ✓ `(3, π/2)` is the correct polar form of (0,3)
  - Q9 (API errors): ✓ status=success
  - Q10 (cost sane): ✓ 2.4k tokens, $0.006
- Verdict: **healthy**

### mbpp-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ "remove first and last occurrence of a character" matches MBPP
  - Q2 (clear): ✓
  - Q3 (format): ✓ "Print ONLY the complete Python code"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✗ first 4 substantive rows are content_filter refusals; never produces a non-refused final
  - Q7 (retry/loops): ⚠ 4 content_filter + 4 max_tokens in 8 rows
  - Q8 (score credible): ✗ if score is recorded as non-zero it's spurious — the fixture never emitted code
  - Q9 (API errors): ⚠ 4 `max_tokens` failures
  - Q10 (cost sane): ✓ 82k tokens, $0.07
- Verdict: **broken** (Q6: four content_filter refusals, no successful code output)

### mgsm-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ Bengali grade-school math matches MGSM
  - Q2 (clear): ✓ standard egg-selling word problem
  - Q3 (format): ✓ "Print only the final numeric answer as a single number"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓
  - Q7 (retry/loops): ✓
  - Q8 (score credible): ✓ answer "18" matches 16-3-4=9 eggs × $2 = $18
  - Q9 (API errors): ✓ status=success
  - Q10 (cost sane): ✓ 13k tokens, $0.03
- Verdict: **healthy**

### mmlu-pro-0-openhands

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ advertising-regulation MCQ matches MMLU-Pro
  - Q2 (clear): ✓
  - Q3 (format): ✓ "Print only the letter of the correct answer"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ answers "I"
  - Q7 (retry/loops): ✓
  - Q8 (score credible): ✓ "I" (Unsafe practices / Distress / Fear / Serious) is the canonical key
  - Q9 (API errors): ⚠ 1 `max_tokens` row then success
  - Q10 (cost sane): ✓ 7k tokens, $0.02
- Verdict: **needs attention** (single transient error; result correct)

### mmmu-0-claude-code

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ cost-accounting question with image matches MMMU's multimodal scope
  - Q2 (clear): ✓
  - Q3 (format): ✓ "Print only the answer letter"
  - Q4 (attachments): ✓ references `/app/image_1.png`
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ⚠ 1 content_filter early, then answer "C"
  - Q7 (retry/loops): ⚠ 2 `max_tokens` failures plus 1 content_filter across 5 rows
  - Q8 (score credible): n.a. — cannot verify without the image table data
  - Q9 (API errors): ⚠ 2 `max_tokens` mid-run
  - Q10 (cost sane): ✓ 61k tokens, $0.06
- Verdict: **needs attention** (content_filter false positive)

### mrcr-0-claude-code

- Mechanical rules: ✓ task (0 findings), ⚠ run (token_runaway 391k, retry_storm)
- Task half:
  - Q1 (domain match): ✓ long-context coreference retrieval matches MRCR v2
  - Q2 (clear): ✗ first user turn is just "Your task is described in the file /app/task.txt — read it and follow the instructions inside." — the real instruction is externalized and the agent has no tool call to read it in the recorded conversation
  - Q3 (format): ✗ format is in the external file, not the prompt
  - Q4 (attachments): ⚠ file is referenced but no tool call to open it; inner-row user messages later contain the actual coreference task (e.g. "prepend `awj4PklzQx` to the response to the **first** 'write a short news article about structures'")
  - Q5 (subtle breakage): ⚠ delegating the prompt to an external file means a mechanical audit that only reads the first user message misses the real task
- Run half:
  - Q6 (attempted task): ⚠ first substantive rows are content_filter refusals, then assistant does emit a real article later
  - Q7 (retry/loops): ✗ 25 rows, 10 `max_tokens` + 8 content_filter — retry_storm confirmed
  - Q8 (score credible): n.a. (coreference check)
  - Q9 (API errors): ⚠ 10 max_tokens mid-run
  - Q10 (cost sane): ✗ 391k tokens, $0.32 — runaway confirmed
- Verdict: **broken** (Q2/Q3 task-half broken — prompt delegates the real instruction to `/app/task.txt` without the agent reading it; Q10 runaway)

### simpleqa-0-goose

- Mechanical rules: ✓ task (0 findings), ⚠ run (retry_storm)
- Task half:
  - Q1 (domain match): ✓ factual QA matches SimpleQA
  - Q2 (clear): ✓
  - Q3 (format): ✓ "Print only the answer, nothing else"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ⚠ goose "Generate a short title…" scaffolding leaks again
- Run half:
  - Q6 (attempted task): ✓ emits a title first, then the answer "Stephen Grossberg"
  - Q7 (retry/loops): ⚠ 7 `max_tokens` failures across 16 rows — mechanical retry_storm corroborated
  - Q8 (score credible): ✗ the 2010 IEEE Frank Rosenblatt Award went to Michio Sugeno, not Stephen Grossberg (Grossberg won in 2017) — a recorded score=1 would be wrong
  - Q9 (API errors): ⚠ 7 `max_tokens` mid-run
  - Q10 (cost sane): ✓ 26k tokens, $0
- Verdict: **broken** (Q8 factually wrong answer for a verifiable SimpleQA item)

### usaco-0-codex

- Mechanical rules: ✓ task (0 findings), ✓ run (0 findings)
- Task half:
  - Q1 (domain match): ✓ "ctiming" (contest timing) matches USACO
  - Q2 (clear): ✓ standard problem statement
  - Q3 (format): ✓ "Print ONLY the complete source code"
  - Q4 (attachments): n.a.
  - Q5 (subtle breakage): ✓
- Run half:
  - Q6 (attempted task): ✓ Python solution computing start and end minute timestamps
  - Q7 (retry/loops): ✓
  - Q8 (score credible): ✓ approach is correct (stop_minute − 11:11 on day 11)
  - Q9 (API errors): ⚠ 1 `max_tokens` failure then success
  - Q10 (cost sane): ✓ 14k tokens, $0.04
- Verdict: **needs attention** (transient API error)

## Summary

- ✓ healthy: 3 (bfcl, math-500, mgsm)
- ⚠ needs attention: 10 (arc-agi, gaia, healthbench, humaneval, ifeval, kumo, livecodebench, mmlu-pro, mmmu, usaco)
- ✗ broken: 10 (aider-polyglot, aime, appworld, browsecomp, gdpval, gpqa-diamond, hle, mbpp, mrcr, simpleqa)

## Top findings

- **Content-filter refusals are the most common silent failure mode.** 7 fixtures (appworld, arc-agi, gdpval, hle, ifeval, mbpp, mmmu, mrcr) contain the literal response "I'm sorry, but I cannot assist with that request." coming from Azure's content filter. `gdpval-0-claude-code` has **13** such refusals across 66 rows; `mbpp-0-claude-code` never recovers past its 4 refusals. The mechanical RUN_RULES never flags these because the refusal is a valid chat completion with `finish_reason=content_filter` — not an `error_str`.
- **`max_tokens` truncation is pervasive and cross-cutting.** 20 of 23 fixtures have at least one row with `litellm.BadRequestError … max_tokens or model output limit was reached`. `gdpval-0-claude-code` row count: 38/66 failures; `gaia-0-goose` 18/37; `hle-0-claude-code` 18/31; `healthbench-0-claude-code` 16/36. These surface as `status=failure` but get retried transparently. The mechanical rules only flag them indirectly via token_runaway (when the sum crosses 200k), missing the shape of the failure entirely.
- **`mrcr-0-claude-code` delegates the real task to `/app/task.txt`** (task row 1 user message: "Your task is described in the file /app/task.txt — read it and follow the instructions inside."). The actual coreference instruction shows up inside inner user messages later in the run. A mechanical audit that only reads the first user message cannot see the real task — this is a task-half brokenness that bypasses the signal catalog.
- **`simpleqa-0-goose` returns a factually wrong answer** that a reader catches but the rules don't: the agent answered "Stephen Grossberg" for the 2010 IEEE Frank Rosenblatt Award (actual winner: Michio Sugeno). Similarly `aime-0-claude-code` answered "43" to an AIME problem whose accepted answer is 116, and `gpqa-diamond-0-codex` picked "D" where the physically-correct answer is "C". These are only catchable by a grader, not by trajectory shape.
- **`gdpval-0-claude-code` references an "attached spreadsheet titled 'Population'" with no file path** anywhere in the prompt. The agent spins up 66 rows, eventually produces partial audit text, but cannot verify anything because the artifact was never handed to it. Combined with the 13 content_filter refusals, this fixture is doubly broken.
- **`browsecomp-0-codex` assumes web access the codex trajectory doesn't have.** 6 of 7 rows are `max_tokens` failures; the one content row ("1988 to 1996") is a bare guess with no browse tool calls visible — the benchmark is mis-plumbed for this agent.

## Mechanical corroboration

- **token_runaway gaia (311k)**: corroborated. 18 of 37 rows are `max_tokens` truncation retries; the run keeps firing new calls with the same goose scaffolding until the budget explodes.
- **token_runaway gdpval (435k)**: corroborated. 66 rows, 38 failures, 13 content_filter refusals — the agent is thrashing against a safety block and a missing attachment, not doing productive work.
- **token_runaway mrcr (391k)**: corroborated. 25 rows, 10 `max_tokens` + 8 content_filter — the long-context task is in an external file so the agent never grounds its work and re-litigates each turn.
- **retry_storm gdpval**: corroborated. Same trace as above, 38 consecutive `max_tokens` retries.
- **retry_storm hle**: corroborated. 18 `max_tokens` + 4 content_filter across 31 rows, with 4 consecutive refusal turns.
- **retry_storm mrcr**: corroborated, as above.
- **retry_storm simpleqa**: corroborated. 7 of 16 rows `max_tokens`; the agent eventually produces a wrong factual answer anyway.

All 7 mechanical yellows are real and supported by reading the traces.

## Rule catalog gap analysis

The walk surfaced several patterns the mechanical RUN_RULES could detect today:

- **new rule `content_filter_refusal`**: any response where `finish_reason == "content_filter"` OR assistant content contains the literal phrase `"I'm sorry, but I cannot assist with that request."` — 7 fixtures currently have this signal invisible to the rules, including `mbpp-0-claude-code` which is wholly refused.
- **new rule `max_tokens_truncation`**: dedicated signal for rows with `error_str` matching `max_tokens or model output limit was reached`, distinct from the generic API-error rule, so operators can tell "the agent's output_tokens cap is too low" from "the proxy is down" — 20 of 23 fixtures would turn yellow, which is actually the correct read.
- **new rule `refusal_final_response`**: the last non-empty assistant content equals the refusal string (applies to `mbpp-0-claude-code`, `appworld-0-claude-code`) — this is a stronger signal than a mid-run refusal because the run never recovered.
- **new rule `task_delegates_to_external_file`**: first user message is under 150 chars and contains a path pattern like `/app/*.txt` or `/tasks/*.md` — would catch `mrcr-0-claude-code`, which bypasses the normal task-half signals by moving the real prompt into a file.
- **new rule `fetch_required_but_no_tool_calls`**: if the task mentions a URL (`http://`, `https://`, `wikipedia`, `arxiv.org`) AND no assistant turn emits a tool call, the agent likely cannot actually browse — would flag `ifeval-0-claude-code` and `browsecomp-0-codex`.
- **new rule `attachment_referenced_but_not_provided`**: task contains `attached spreadsheet`, `the attached`, `see /app/*.png`, `image data in /app/` etc., and that exact path does not appear in any assistant tool call — would flag `gdpval-0-claude-code` and `hle-0-claude-code`.
