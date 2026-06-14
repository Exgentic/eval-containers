---
name: audit-trajectory
description: >-
  Run a judgment-level health review of a recorded evaluation trajectory in two
  halves — was the task the agent saw well-formed (the prompt), and was the
  actual run sane (refusals, retries, wrong answers, API errors, cost). This is
  the procedural layer that catches content-filter refusals, factually wrong
  answers on verifiable benchmarks, and tasks delegated to an external file that
  the mechanical trajectory rule catalog cannot see. Use this for "audit the
  fixtures/trajectories", when the mechanical catalog flags a yellow, when a new
  benchmark batch lands, or as step 24 of the `verify` release walk. This is the
  per-fixture pass; for per-Dockerfile static health use audit-dockerfile, and
  for the whole-repo sweep use audit-fleet.
---

# Audit a trajectory's health

Structural tests (compose parses, Dockerfile builds) tell you the benchmark is
**well-formed**. They do not tell you it is **legitimate**, and they do not tell
you the agent's actual run was **sane**. A benchmark can build perfectly, hand
the agent an empty task, and produce a valid-looking `result.json` that scores
zero. An agent can be handed a perfect task, burn 200k tokens in a retry storm,
hit an API error, and produce garbage while still reporting a plausible score.

This audit closes both gaps. It has **two halves**:

1. **Task half** — was the prompt the agent saw well-formed? Read from the first
   non-empty user message in the trajectory.
2. **Run half** — was the actual conversation sane? Read from every row: API
   status, tokens, cost, retries, tool errors, empty responses.

The overall fixture verdict is the **worse of the two halves**. The audit is
toolchain-agnostic: a human walks it with `less` and a notepad, a sub-agent
reads this checklist and executes it, a script implements the mechanizable
parts. The output format is fixed so findings are comparable across releases.

The full two-half signal catalog (task-half + run-half red / yellow / green
signals), the collection mechanism, and the four-layer checking model are bulky
reference material; they live in `references/checklist.md` beside this skill.
Read it before walking the ten questions so you know what the mechanical layer
already covers.

## Rules this skill serves

- `tests/sanity/RULES.md:6` — the data-driven trajectory rule
  catalog applied to every fixture is the mechanical layer this audit sits on
  top of; the audit finds what those rules miss.
- `tests/replay/RULES.md:7` — `fixtures/broken.json` marks
  fixtures whose recorded run is known-bad (refusals, wrong answers, content
  filter, max-tokens truncation); findings on those are reported but do not
  fail. This audit is how a fixture earns a broken-manifest entry.
- `tests/live/RULES.md:9-12` — every live run must pass the
  trajectory rule catalog before its fixture is promoted; a red finding blocks
  promotion and a yellow annotates it. This audit is the human/agent half of
  that gate, and its red/yellow split mirrors the live trace-inspection
  checklist (`tests/live/RULES.md:13-33`).
- `.agents/verification/RULES.md:13` — mechanical > procedural > aspirational;
  this is the procedural tier, not a substitute for the mechanical catalog.

## Procedure

For each `tests/replay/fixtures/*.trajectory.jsonl` (or a directory of live
`inspector` outputs):

1. **Gather context.** Note the benchmark name, its
   `eval.benchmark.description`, the task ID, and the agent name. WHY: question 1
   (domain match) and question 8 (score credibility) can only be judged against
   what the benchmark claims to test.

2. **Extract the task.** Read rows until the first row with a non-empty user
   message; concatenate every `role: "user"` message in that row into the task
   text. Ignore system and developer messages — they are framework scaffolding.
   WHY: the task half is judged against the prompt the agent actually saw.

3. **Run the mechanical catalog first.** `cargo test --test check trajectory`
   (or the live `inspect_trajectory` driver). Note any findings. WHY: the
   audit's job is to find what the rules missed
   (`tests/sanity/RULES.md:6`).

4. **Walk the task half** — five questions about the first user message, each
   yes / no / n.a. with a one-line reason. WHY: a malformed prompt makes any
   score meaningless regardless of how the run went.

   | # | Question |
   |---|----------|
   | 1 | Does the task match the benchmark's stated domain? |
   | 2 | Is the instruction clear enough for a competent human? |
   | 3 | Is the expected output format obvious from the prompt? |
   | 4 | If the benchmark needs attached files (images, docs, repos), does the prompt reference them? |
   | 5 | Any subtle signs of a broken environment you would only catch by reading (dangling references, contradictory instructions, wrong task ID in the prompt)? |

5. **Walk the run half** — scroll through every assistant turn and tool call,
   five questions about the conversation as a whole. WHY: a well-formed task can
   still produce a broken run (refusal, retry storm, wrong answer, API failure).

   | # | Question |
   |---|----------|
   | 6 | Did the agent's first substantive response actually attempt the task, or refuse / stall / ask for clarification? |
   | 7 | Are there repeated tool errors, retry storms, or identical messages sent multiple times in a row? |
   | 8 | Does the assistant's final answer match the recorded score? (score=1 → is the answer actually correct? score=0 → do you see why it failed?) |
   | 9 | Any API errors, context overflows, or auth/network failures mid-run? |
   | 10 | Is the cost / token count reasonable for this task, or did the agent burn resources in a loop? |

6. **Classify each half, then take the worse.** WHY: questions 1/5 (task) and
   6/8/9 (run) are correctness; 2/3/4 (task) and 7/10 (run) are fixable
   quality.
   - **healthy** — all yes or n.a. in that half.
   - **needs attention** — task-half Q2/Q3/Q4 no, or run-half Q7/Q10 no.
   - **broken** — task-half Q1/Q5 no, or run-half Q6/Q8/Q9 no.
   - **Overall fixture verdict = the worse of the two halves.**

7. **Emit one report entry per fixture** in the fixed format, then a summary
   count and suggested fixes. WHY: the fixed shape lets findings diff across
   releases.

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

8. **Record broken fixtures in the manifest.** A fixture that comes up broken is
   recorded in `tests/replay/fixtures/broken.json` (not deleted); a fixture from
   a live run that fails an inspection rule is not promoted and goes to
   `tests/live/known-broken.md` with the rule that tripped. WHY: this is the
   broken-fixture contract (`tests/replay/RULES.md:7`,
   `tests/live/RULES.md:12`) — known-bad fixtures are documented
   and re-recorded next cycle, not silently dropped.

9. **Propose catalog rules for any mechanical-shaped finding.** If a smell
   recurs and could be a check (e.g. `finish_reason == "content_filter"` or the
   literal refusal string, a `max_tokens` truncation signal, a first user
   message under 150 chars pointing at `/app/*.txt`, a task that names an
   attachment whose path never appears in a tool call), record it as a proposed
   new rule for the catalog. WHY: a recurring manual finding belongs in code
   (`tests/sanity/RULES.md:9`).

## When to run

- Before cutting a release (whole fleet) — step 24 of the `verify` skill.
- When the mechanical trajectory catalog flags a yellow that needs judgment.
- When a new benchmark batch lands.
- After each live run, as the human/agent half of fixture promotion
  (`tests/live/RULES.md:9`).
- Quarterly, as a health check.

## References

- `references/checklist.md` — the full task-half and run-half signal catalogs,
  classification rules, collection mechanism (the inspector model), layered
  model, and output format.
- `tests/sanity/RULES.md` — the mechanical trajectory rule
  catalog this audit complements.
- `tests/live/RULES.md:13-33` — the live trace-inspection
  checklist whose red/yellow split this audit mirrors.
- `.agents/verification/audit-dockerfile/SKILL.md` — the parallel per-file
  static-health audit.
