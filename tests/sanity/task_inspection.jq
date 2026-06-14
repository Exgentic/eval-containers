# Trajectory health inspection — extraction + rule engine.
#
# Framework-free port of tests/sanity/task_inspection.rs. The Rust file
# carries the authoritative doc comment and rule rationale; this program
# is the mechanical half. It is invoked two ways (see task_inspection.bats
# and task_inspection_audit.release.bats):
#
#   1. Per-fixture audit. Read one *.trajectory.jsonl as raw text:
#          jq -rR -f task_inspection.jq --arg source NAME --slurp '<text>'
#      with $mode "findings" (default). Emits one TSV line per finding:
#          <severity>\t<source>\t<rule>\t<why>
#      severity is "red" or "yellow"; rule is the catalog id; why is the
#      catalog rationale. No findings → no output.
#
#   2. Rule unit tests. Feed a synthetic task string or synthetic run
#      summary and ask which rule ids fire (--arg mode task_rules /
#      run_rules / summary), so the bats unit tests can drive one rule at
#      a time with hand-built inputs exactly like the Rust #[test]s.
#
# CRUCIAL invariants preserved from the Rust:
#   * Lengths are CHAR (Unicode scalar) counts, never bytes. jq `length`
#     on a string is the Unicode codepoint count — the same thing Rust's
#     `chars().count()` measures. Never shell out to `wc -c`/`wc -m` for
#     these (the test host's locale makes `wc -m` == `wc -c` == bytes).
#   * Assistant content is folded over BOTH response shapes:
#       - Responses API:  .response.output[].content[].text  +  .output[].text
#       - Chat Completions: .response.choices[].message.content  + .tool_calls
#     Missing either shape silently undercounts substantive rows.
#   * Malformed JSONL lines are skipped per-line (mirrors the Rust
#     `serde_json::from_str(l).ok()` filter) — NOT a blanket 2>/dev/null.
#   * `fetch_failed` matching is literal substring, so it fires its known
#     false positive on webarena-486-plandex ("404 Not Found" in the real
#     task). That behavior is preserved on purpose.

# ── helpers ────────────────────────────────────────────────────────────

def lc: ascii_downcase;

# True if string s contains ANY of the substrings in list a.
def contains_any($a): . as $s | any($a[]; . as $n | $s | contains($n));

# Whole-token match: does the text contain $tok as a maximal run of
# alphanumerics? Mirrors the Rust `split(|c| !c.is_alphanumeric())` then
# `w == tok`. jq's \p{L}\p{N} class is Unicode-aware like Rust's
# is_alphanumeric(). Empty tokens (from runs of separators) never equal a
# non-empty $tok, so they are harmless.
def has_token($tok):
  . as $s
  | ([$s | scan("[\\p{L}\\p{N}]+")] | any(. == $tok));

# Count non-overlapping LITERAL occurrences of $n in . — the exact
# semantics of Rust's str::matches(...).count() (advance past each hit, no
# regex). Used by repeated_block. jq's `index` works on Unicode codepoint
# offsets, so this is char-correct on multibyte (CJK) text, and treating
# the needle literally means a block full of regex metacharacters (code,
# markdown, punctuation) is matched the same way the Rust does — a regex
# `splits` would mis-handle those.
def count_lit($n):
  if ($n | length) == 0 then 0
  else
    def go($s; $acc):
      ($s | index($n)) as $i
      | if $i == null then $acc
        else go($s[$i + ($n | length):]; $acc + 1)
        end;
    go(.; 0)
  end;

# ── task-half extraction (first legitimate user turn) ──────────────────
#
# Walk a row's messages; concat every user message's text. String content
# is taken verbatim; array content contributes each part's .text or
# .input_text. Joined with "\n\n" (matches Rust parts.join("\n\n")).
def user_text_from_row:
  (.messages // []) as $msgs
  | [ $msgs[]
      | select(.role == "user")
      | (.content // null) as $c
      | if   ($c | type) == "string" then $c
        elif ($c | type) == "array"  then
          ($c[] | (.text // .input_text // empty))
        else empty end
    ]
  | join("\n\n");

# First row (in file order) whose user text is non-empty after trim.
# Mirrors extract_user_text_from_fixture: the "first legitimate user turn".
def first_user_text($rows):
  [ $rows[]
    | user_text_from_row
    | select((. | gsub("^\\s+|\\s+$"; "")) != "")
  ]
  | (first // "");

# ── run-half extraction (per row) ──────────────────────────────────────

# Fold assistant content over both response shapes into one string; a
# non-empty (trimmed) result means the row was "substantive". Tool calls
# count as substantive output (Rust pushes a "<tool_calls>" sentinel).
def assistant_content_from_row:
  (.response // null) as $r
  | if $r == null then "" else
      ( # Shape 1: Responses API — text nested under content, and text
        # placed directly on the output item. Each branch is fully
        # parenthesized: jq's `|` binds looser than `,`, so an un-parenthesized
        # comma branch would re-apply the next pipeline to the previous
        # branch's output and index a string.
        [ ($r.output // [])[] | (.content // [])[] | (.text // empty) ]
        + [ ($r.output // [])[] | (.text // empty) ]
        # Shape 2: Chat Completions — assistant text, plus a sentinel when
        # the row carries tool_calls (tool calls count as substantive).
        + [ ($r.choices // [])[] | .message.content | select(type == "string") ]
        + [ ($r.choices // [])[]
            | (.message.tool_calls // []) | select(length > 0) | "<tool_calls>" ]
      ) | join("\n")
    end;

# Every error signal LiteLLM records on a row, joined with " | ". Mirrors
# row_error_message: error_str (skip "" and "None"), the four
# error_information string fields, and response.error (string or
# stringified non-null object).
def row_error_message:
  ( [ (.error_str // empty) | select(. != "" and . != "None") ]
    + [ (.error_information // {}) as $e
        | ("error_code","error_class","error_message","traceback")
        | $e[.] // empty | select(type == "string" and . != "") ]
    + [ (.response.error // null) as $err
        | if   ($err | type) == "string" then (select($err != "") | $err)
          elif $err == null then empty
          else ($err | tojson) end ]
  ) | join(" | ");

# finish/stop-reason truncation across all three shapes.
def row_hit_max_tokens:
  (.response // {}) as $r
  | ( ($r.incomplete_details.reason // "")
        | (. == "max_output_tokens" or . == "max_tokens" or . == "length") )
    or ( ($r.choices // []) | any(.finish_reason == "length") )
    or ( ($r.stop_reason // "") == "max_tokens" );

# tool_calls present in either shape.
def row_has_tool_calls:
  (.response // {}) as $r
  | ( ($r.choices // []) | any((.message.tool_calls // []) | length > 0) )
    or ( ($r.output // []) | any(
           (.type // "") as $t
           | $t == "function_call" or $t == "tool_use" or $t == "tool_call") );

# ── refusal / heuristic phrase tables (verbatim from the Rust) ─────────

def refusal_phrases:
  [ "i'm sorry, but i cannot assist",
    "i'm sorry, but i can't assist",
    "i cannot help with that request",
    "i cannot provide assistance",
    "i cannot comply",
    "unable to assist with that request",
    "against my guidelines",
    "i must refuse" ];

def content_is_refusal: (lc) | contains_any(refusal_phrases);

# Short message pointing at an in-container path with a file verb.
def task_delegates_to_file_heuristic:
  (gsub("^\\s+|\\s+$"; "")) as $t
  | if ($t | length) > 400 then false
    else ($t | lc) as $l
      | ($l | contains_any(["/app/","/tasks/","/workspace/","/data/"]))
        and ($l | contains_any(["read","see","open","load"]))
    end;

def task_references_attachment_heuristic:
  (lc) | contains_any([
    "attached spreadsheet","attached document","attached image",
    "attached file","the attached","the uploaded","uploaded file",
    "see the image","see the spreadsheet","refer to the attached" ]);

def task_requires_fetching_heuristic:
  (lc) as $l
  | ($l | contains_any([
      "search the web","look up","browse to","visit the",
      "open the following url","fetch the page","scrape" ]))
    or ($l | contains("http://")) or ($l | contains("https://"));

# ── run summary (one pass over rows, mirrors summarize_run) ────────────

def summarize_run($rows):
  ($rows | length) as $n_rows
  # Per-row derived records, in file order.
  | [ $rows[]
      | { status: (.status // ""),
          tokens: (.total_tokens // 0),
          cost: (.response_cost // 0),
          err: row_error_message,
          maxtok: row_hit_max_tokens,
          tools: row_has_tool_calls,
          assistant: assistant_content_from_row,
          prompt: user_text_from_row }
      | .substantive = ((.assistant | gsub("^\\s+|\\s+$"; "")) != "")
    ] as $r
  | [ $r[] | select(.substantive) ] as $sub
  # Retry-storm: longest run of identical non-empty adjacent prompts.
  # reduce mirrors the Rust streak walk exactly (only non-empty prompts
  # advance/compare; last_prompt only updates on a non-empty mismatch).
  | ( reduce $r[] as $row
        ( {last:"", cur:1, max:1};
          if ($row.prompt | length) > 0 then
            if $row.prompt == .last then
              .cur += 1 | (if .cur > .max then .max = .cur else . end)
            else .cur = 1 | .last = $row.prompt end
          else . end )
      | .max ) as $max_streak
  | ( [ $r[] | select(.err != "") ] | (first.err // "") ) as $any_err
  | ( $sub | last ) as $last_sub
  | { n_rows: $n_rows,
      n_substantive_rows: ($sub | length),
      n_failure_rows:
        ([ $sub[] | select(.status != "success" and .status != "") ] | length),
      last_substantive_status: ($last_sub.status // ""),
      any_assistant_content_nonempty: (($sub | length) > 0),
      total_tokens: ([ $r[].tokens ] | add // 0),
      total_cost: ([ $r[].cost ] | add // 0),
      max_consecutive_identical_prompts: $max_streak,
      any_error_message: $any_err,
      n_refusal_rows:
        ([ $sub[] | select(.assistant | content_is_refusal) ] | length),
      n_max_tokens_rows: ([ $r[] | select(.maxtok) ] | length),
      final_response_is_refusal:
        (($last_sub != null)
         and (($last_sub.assistant) != "")
         and ($last_sub.assistant | content_is_refusal)),
    }
  | ( first_user_text($rows) ) as $first
  | .task_delegates_to_file = ($first | task_delegates_to_file_heuristic)
  | .task_references_attachment = ($first | task_references_attachment_heuristic)
  | .fetch_required_but_no_tool_calls =
      (($first | task_requires_fetching_heuristic)
       and (([ $r[] | select(.tools) ] | length) == 0));

# ── caps ───────────────────────────────────────────────────────────────
def COST_CAP_USD: 5.0;
def TOKEN_CAP: 200000;
def TURN_CAP: 100;

# ── task-half rule catalog ─────────────────────────────────────────────
# . is the task string. Each entry: {id, sev, why, fire}. `fire` is a
# boolean computed against the task. Order preserved from the Rust RULES.
def task_rules:
  . as $t
  | ($t | gsub("^\\s+|\\s+$"; "")) as $tt
  | [
      { id:"empty", sev:"red",
        why:"user message is empty or whitespace",
        fire: ($tt == "") },
      { id:"env_leaked", sev:"red",
        why:"unresolved EVAL_* env var in task (substitution failed)",
        fire: ($t | contains_any(
          ["$EVAL_BENCHMARK","${EVAL_BENCHMARK}","$EVAL_TASK_ID",
           "${EVAL_TASK_ID}","${TASK}"])) },
      { id:"template_leak", sev:"red",
        why:"TEMPLATE.md placeholder leaked into task (author forgot to fill in)",
        fire: ($t | contains_any(
          ["{NAME}","{TASK_PROMPT}","{DATASET}","{SPLIT}",
           "{QUESTION_FIELD}","{ANSWER_FIELD}","{ID_FIELD}"])) },
      { id:"fetch_failed", sev:"red",
        why:"task contains evidence of a failed dataset download",
        fire: ($t | contains_any(
          ["404 Not Found","403 Forbidden","HF_TOKEN required",
           "access denied","401 Unauthorized"])) },
      { id:"file_missing", sev:"red",
        why:"task contains filesystem errors",
        fire: (($t | lc) | contains_any(
          ["no such file or directory","permission denied",
           "cannot open","not a directory"])) },
      { id:"unresolved_url_var", sev:"red",
        why:"task contains a URL with an unsubstituted shell var — fetch returned literal",
        fire: (($t | contains("${"))
               and (($t | contains("http://")) or ($t | contains("https://")))) },
      { id:"todo_or_fixme", sev:"red",
        why:"task definition contains TODO/FIXME/XXX — unfinished",
        fire: (($t | has_token("TODO")) or ($t | has_token("FIXME"))
               or ($t | has_token("XXX"))) },
      { id:"control_garbage", sev:"red",
        why:"task contains non-printable control chars (encoding corruption)",
        # Control chars are codepoints < 0x20 or 0x7f (DEL), excluding
        # \n \t \r. Mirrors Rust char::is_control() && not in {\n,\t,\r}.
        fire: ([ $t | explode[]
                 | select((. < 32 or . == 127)
                          and . != 10 and . != 9 and . != 13) ]
               | length > 0) },
      # ── yellow ──
      { id:"too_short", sev:"yellow",
        why:"task text < 20 chars — almost certainly a template miss",
        fire: ($tt != "" and ($tt | length) < 20) },
      { id:"borderline_short", sev:"yellow",
        why:"task is 20-50 chars — suspicious, worth a human glance",
        fire: (($tt | length) >= 20 and ($tt | length) < 50) },
      { id:"runaway_long", sev:"yellow",
        why:"> 50k chars — possible template concat runaway",
        fire: (($t | length) > 50000) },
      { id:"repeated_block", sev:"yellow",
        why:"same 200-char block repeats 10+ times — possible concat runaway",
        # Cheap heuristic, char-indexed exactly like the Rust: for each of
        # the probe offsets 0/200/400/600/800, take the 200-CHAR window and
        # count its non-overlapping literal occurrences; fire if any window
        # appears >= 10 times. Short inputs (< 2000 chars) skip the check.
        # `explode`/`implode` index by codepoint so a window never lands
        # mid-codepoint (which would panic the byte-sliced version).
        fire: (
          if ($t | length) < 2000 then false
          else ($t | explode) as $ch
            | any( (0,200,400,600,800);
                   . as $start
                   | ($start + 200) <= ($ch | length)
                   and (($ch[$start:$start+200] | implode) as $probe
                        | ($t | count_lit($probe)) >= 10) )
          end) },
      { id:"no_instruction_verb", sev:"yellow",
        why:"no instruction verb (solve/write/compute/answer/translate/find/explain/return/print/select)",
        fire: (($t | lc) | (contains_any(
          ["solve","write","compute","answer","translate","find",
           "explain","return","print","select","complete","analyze",
           "identify","classify","generate","implement","describe",
           "summarize"]) | not)) }
    ];

# ── run-half rule catalog ──────────────────────────────────────────────
# . is the run summary object. Order preserved from the Rust RUN_RULES.
def run_rules:
  . as $s
  | [
      { id:"no_substantive_output", sev:"red",
        why:"every LLM call produced zero content and zero tool calls — the run said nothing",
        fire: ($s.n_rows > 0 and ($s.any_assistant_content_nonempty | not)) },
      { id:"last_substantive_row_failed", sev:"red",
        why:"the final LLM call that produced real output ended in status != success",
        fire: ($s.n_substantive_rows > 0 and $s.last_substantive_status != "success") },
      { id:"context_overflow", sev:"red",
        why:"context window was exceeded",
        fire: (($s.any_error_message | lc) | contains_any(
          ["context_length_exceeded","context window","maximum context length"])) },
      { id:"auth_failure", sev:"red",
        why:"an LLM call hit an auth/permission error (401/403/invalid key)",
        fire: (($s.any_error_message | lc) | contains_any(
          ["401 unauthorized","403 forbidden","invalid api key",
           "authenticationerror","permission denied"])) },
      # ── yellow ──
      { id:"cost_runaway", sev:"yellow",
        why:"total response_cost exceeds the per-task cap ($5)",
        fire: ($s.total_cost > COST_CAP_USD) },
      { id:"token_runaway", sev:"yellow",
        why:"total_tokens exceed the per-task cap (200k)",
        fire: ($s.total_tokens > TOKEN_CAP) },
      { id:"high_turn_count", sev:"yellow",
        why:"more than 100 LLM calls for a single task",
        fire: ($s.n_rows > TURN_CAP) },
      { id:"retry_storm", sev:"yellow",
        why:"same prompt repeated 5+ times in a row with no material change",
        fire: ($s.max_consecutive_identical_prompts >= 5) },
      { id:"high_substantive_failure_rate", sev:"yellow",
        why:"more than half of the substantive LLM calls ended in failure",
        fire: ($s.n_substantive_rows > 1 and ($s.n_failure_rows * 2) > $s.n_substantive_rows) },
      # ── 2026-04-15 audit additions ──
      { id:"refusal_final_response", sev:"red",
        why:"the final substantive assistant turn is a safety refusal — the run never answered the task",
        fire: ($s.final_response_is_refusal) },
      { id:"content_filter_refusal", sev:"yellow",
        why:"one or more assistant turns contain a content_filter refusal (rides a valid response body)",
        fire: ($s.n_refusal_rows > 0) },
      { id:"max_tokens_truncation", sev:"yellow",
        why:"one or more assistant turns were truncated at max_tokens mid-answer",
        fire: ($s.n_max_tokens_rows > 0) },
      { id:"task_delegates_to_external_file", sev:"red",
        why:"first user message is a short pointer to a file (e.g. /app/task.txt) — the task-half rule catalog never saw the real instruction",
        fire: ($s.task_delegates_to_file) },
      { id:"attachment_referenced_but_not_provided", sev:"yellow",
        why:"task mentions an attached file / spreadsheet / image / document but no file path is provided",
        fire: ($s.task_references_attachment) },
      { id:"fetch_required_but_no_tool_calls", sev:"yellow",
        why:"task requires browsing / searching / fetching a URL but the trace has zero tool_calls",
        fire: ($s.fetch_required_but_no_tool_calls) }
    ];

# ── line parsing (explicit per-line skip, mirrors the Rust filter_map) ──
# Input arrives as one raw string (jq -R --slurp). Split on newlines,
# drop whitespace-only lines, and parse each remaining line. A line that
# fails to parse is skipped — the SAME per-line tolerance the Rust has
# (`serde_json::from_str(l).ok()` then filter_map), scoped to one line,
# not a blanket error redirect.
def parse_rows:
  [ split("\n")[]
    | select((gsub("^\\s+|\\s+$"; "")) != "")
    | (try fromjson catch empty) ];

# ── entry point ────────────────────────────────────────────────────────
# Callers MUST pass --arg mode <m> and --arg source <s> (the latter may be
# ""). jq compiles every branch, so $source must be defined even when the
# selected mode does not use it. The bats harnesses always pass both.
#   $mode:
#     findings    — input is raw fixture text; emit TSV findings
#                   (severity\tsource\trule\twhy).
#     task_rules  — input is raw task text; emit ids of firing task rules.
#     run_rules   — input is raw JSON run-summary; emit ids of firing run rules.
#     refusal     — input is raw assistant text; emit "true"/"false"
#                   (exposes content_is_refusal for the heuristic unit test).
#     delegates   — input is raw task text; emit "true"/"false"
#                   (exposes task_delegates_to_file_heuristic).
#     summary     — input is raw fixture text; emit the run-summary JSON.
#     first_user  — input is raw fixture text; emit the first user text.
$mode as $m
| if $m == "task_rules" then
    ( task_rules | .[] | select(.fire) | .id )
  elif $m == "run_rules" then
    ( fromjson | run_rules | .[] | select(.fire) | .id )
  elif $m == "refusal" then
    ( content_is_refusal )
  elif $m == "delegates" then
    ( task_delegates_to_file_heuristic )
  elif $m == "summary" then
    ( parse_rows as $rows | summarize_run($rows) )
  elif $m == "first_user" then
    ( parse_rows as $rows | first_user_text($rows) )
  else
    # findings: task half then run half, TSV, severity\tsource\trule\twhy
    parse_rows as $rows
    | ( ( first_user_text($rows) | task_rules )
        + ( summarize_run($rows) | run_rules ) )
    | .[]
    | select(.fire)
    | [ .sev, $source, .id, .why ] | @tsv
  end
