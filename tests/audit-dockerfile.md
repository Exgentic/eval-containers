# Dockerfile Audit — 2026-04-15

Commit: `90244c2`
Walked by: procedural audit per tests/DOCKERFILE.md
Sample: 12 of 113 Dockerfiles (6 yellow findings + 6 random clean)

## Per-file verdicts

### benchmarks/appworld/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — `ghcr.io/stonybrooknlp/appworld:latest`)
- Q1 (install order): ✓ base → apt curl → env → pyarrow → fetch → uninstall pyarrow → entrypoint; sane
- Q2 (comments): ✓ header explains upstream base, state-based scoring, and the -1 externally-graded contract
- Q3 (dead code): ✓ no unused ARG, no commented-out blocks
- Q4 (bloat): ⚠ inherits full AppWorld image incl. all 9 simulated apps; intentional but the image is large and unavoidable
- Q5 (labels accurate): ✓ labels match; data_revision=`refs/convert/parquet` matches the HF URL on L30
- Q6 (entrypoint sane): ✓ handles missing `DOCK_TASK_ID`, falls through to shared entrypoint cleanly
- Q7 (subtle smells): ⚠ L23 sets `ENV DOCK_BENCHMARK=appworld` *after* the label block; convention elsewhere is label-then-env together. Minor.
- Verdict: healthy (yellow mechanical flag is unavoidable; upstream doesn't publish immutable tags)

### benchmarks/cybench/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — per-task `:latest`)
- Q1 (install order): ✓ ARG → FROM → labels → env → apt → task fetch → tests → entrypoint
- Q2 (comments): ✓ FUTURE block at top explains the per-task image convention and why the base path is tentative
- Q3 (dead code): ✗ L30 declares `ARG DOCK_TASK_ID` a SECOND time after L12. The first is needed pre-FROM; the second is superfluous because `ENV DOCK_TASK_ID` already captures it. Same pattern in swe-bench, swe-bench-pro, swe-lancer, mle-bench.
- Q4 (bloat): ✓ only curl/jq/git added beyond the upstream per-task base
- Q5 (labels accurate): ✓ name=cybench matches dir; upstream_base interpolates `${DOCK_TASK_ID}` correctly
- Q6 (entrypoint sane): ✓ chowns /app to agent, falls through, handles missing task
- Q7 (subtle smells): ✗ L39–40 bake literal `"TODO"` strings into `/tasks/0/problem.txt` and `/tasks/0/answer.txt` as *data* when the upstream JSON is missing fields. The image will silently ship and score a task whose expected answer is the string `"TODO"`. Should `exit 1` at build time, not fall through with TODO data. The mechanical TODO-in-comment rule does not catch this (it's inside a string literal). Same shape in mle-bench L41, swe-lancer L38/L40.
- Verdict: needs attention (Q3 dead ARG and Q7 TODO-as-data are fixable)

### benchmarks/mle-bench/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — per-task `:latest`)
- Q1 (install order): ✓ ARG → FROM → labels → apt → task → tests → entrypoint
- Q2 (comments): ✓ FUTURE block honestly explains Kaggle creds + upstream pinning gap
- Q3 (dead code): ✗ double `ARG DOCK_TASK_ID` (L13, L30), same smell as cybench
- Q4 (bloat): ✓ `--target /tests/deps` keeps grader libs out of the agent's PYTHONPATH; good pattern
- Q5 (labels accurate): ✓
- Q6 (entrypoint sane): ✓ creates /home/submission, chowns, builds TASK from problem.txt
- Q7 (subtle smells): ⚠ L44 `pip install ... mlebench==0.1.0 2>/dev/null || true` — the `|| true` means a failed install is silently swallowed and grade.py will ImportError at test time, writing 0. Should be `set -e` style. Plus the L41 `"TODO: MLE-bench task description..."` data-fallback (same shape as cybench Q7).
- Verdict: needs attention

### benchmarks/swe-bench/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — per-task `:latest`)
- Q1 (install order): ⚠ `pip install swebench` on L25 happens BEFORE `apt-get install curl jq` on L29. apt-get layer change will invalidate the pip layer on rebuild. Flip the order: apt first, pip second.
- Q2 (comments): ✓ inline comments explain root-only /tests, grader responsibilities, conda activation
- Q3 (dead code): ✗ double `ARG DOCK_TASK_ID` (L6, L30)
- Q4 (bloat): ✓ grader pinned to `/tests/deps`, isolated from agent
- Q5 (labels accurate): ✓ `data_revision=c104f840...` is a real SHA; good example of green label
- Q6 (entrypoint sane): ✓ chowns /testbed, wraps agent in TASK
- Q7 (subtle smells): ⚠ L25 `pip install ... 2>/dev/null || pip3 install ...` — swallows stderr and silently falls through. If both fail the grader deps are missing; test.sh logic will produce 0 instead of a build failure. Real bug potential.
- Verdict: needs attention (L25 ordering + silent-fail)

### benchmarks/swe-bench-pro/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — per-task `:latest`)
- Q1 (install order): ⚠ same problem as swe-bench: pip install (L27) before apt-get (L30); layer cache suboptimal
- Q2 (comments): ✓ FUTURE block explains the tentative registry namespace
- Q3 (dead code): ✗ double `ARG DOCK_TASK_ID` (L10, L31)
- Q4 (bloat): ✓ `/tests/deps` isolation preserved
- Q5 (labels accurate): ⚠ missing `dock.benchmark.data_revision` label entirely, where swe-bench has one. If the Pro dataset has a published commit, it should be pinned here. If it doesn't, the absence should be documented.
- Q6 (entrypoint sane): ✓
- Q7 (subtle smells): ⚠ same silent `2>/dev/null || pip3 install` swallowing on L27; same grade.py structure as swe-bench means same failure modes
- Verdict: needs attention

### benchmarks/swe-lancer/Dockerfile

- Mechanical rules: ⚠ (upstream_base_unpinned — per-task `:latest`)
- Q1 (install order): ✓ apt → env → task fetch → test scripts → entrypoint
- Q2 (comments): ✓ FUTURE block is clear about upstream registry uncertainty
- Q3 (dead code): ✗ double `ARG DOCK_TASK_ID` (L10, L27)
- Q4 (bloat): ✓ minimal apt additions
- Q5 (labels accurate): ✓ labels consistent with dir
- Q6 (entrypoint sane): ✓ tries /app/expensify then /workspace
- Q7 (subtle smells): ✗ L40 `printf '#!/bin/bash\necho "TODO: upstream run_tests.sh not found"\nexit 1\n' > /tests/run_tests.sh` installs a placeholder script that always fails — but the test.sh at L51–55 treats any non-zero as reward=0. So a task whose upstream image lacks `run_tests.sh` will silently score 0 instead of erroring out. Also L38 same TODO-as-data pattern.
- Verdict: needs attention

### benchmarks/aime/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓ slim base → apt curl → pyarrow → fetch → uninstall pyarrow → lockdown → entrypoint
- Q2 (comments): ✓ short header states everything a maintainer needs
- Q3 (dead code): ✓ none
- Q4 (bloat): ✓ transient pyarrow pattern; `pip uninstall -y pyarrow` at L31 removes it post-fetch. Note: this doesn't reclaim space in the previous layer — `pip install` at L20 still lives in a prior layer. Squashing would help but slim base keeps total size reasonable.
- Q5 (labels accurate): ✓ data_revision SHA matches a concrete HF commit
- Q6 (entrypoint sane): ✓ sets TASK and EXPECTED_ANSWER from task files
- Q7 (subtle smells): ⚠ `pip uninstall -y pyarrow` on L31 is theatre — the earlier `RUN pip install pyarrow` layer still contains the package. Image size is unchanged. Merge L20/L21/L31 into one RUN to actually strip pyarrow. Same pattern in appworld, humaneval, ifeval, gdpval — it's fleet-wide.
- Verdict: healthy (Q7 is an optimization, not a bug)

### benchmarks/humaneval/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓
- Q2 (comments): ✓ header states benchmark + format, section comments are clear
- Q3 (dead code): ✓
- Q4 (bloat): ⚠ same phantom pyarrow uninstall as aime (L22/L23/L36 in separate RUNs)
- Q5 (labels accurate): ✓ SHA in data_revision
- Q6 (entrypoint sane): ✓
- Q7 (subtle smells): ⚠ test.sh at L54–62 heredoc-interpolates `$SOLUTION` directly into a Python file. If the agent's stdout contains a `PYEOF` token or a Python triple-quote it breaks the test harness. Risk is low (heredoc tag is `PYEOF`) but a quoted-eof or a subprocess with the input on stdin would be more robust.
- Verdict: healthy

### benchmarks/ifeval/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓ base → apt → pyarrow → fetch → official deps → nltk data → cloned verifier → lockdown
- Q2 (comments): ✓ section comments explain each block's purpose
- Q3 (dead code): ⚠ L111–157 define `test_instruction_following_loose` which is then never called (L166 uses the strict variant only). Dead code inside a heredoc. Real finding the mechanical rules miss.
- Q4 (bloat): ⚠ full google-research instruction_following_eval copied to /opt — also fine. pyarrow phantom uninstall pattern again (L22/L48).
- Q5 (labels accurate): ✓ data_revision SHA present
- Q6 (entrypoint sane): ✓ uses prompt.txt, no EXPECTED_ANSWER because scoring is verifier-based
- Q7 (subtle smells): ✓ aside from Q3's dead `_loose` function
- Verdict: needs attention (Q3 dead helper)

### benchmarks/gdpval/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ⚠ libreoffice installed at L18 before Python deps. Fine for cache. But huggingface_hub (L23) is installed before pyarrow (L26) in *separate* RUN layers, both without version pins. `pip install huggingface_hub[cli]` on L23 is genuinely unpinned (not transient — it's used at runtime inside test.sh) and should be flagged RED by the unpinned_pip rule. Worth checking whether the rule allowlist covers `huggingface_hub` as transient (it shouldn't — it ships in the final image and the entrypoint calls `huggingface-cli`).
- Q2 (comments): ✓
- Q3 (dead code): ✓
- Q4 (bloat): ✗ `libreoffice` is ~1 GB of headless office. It's installed on L18 but nothing in the Dockerfile ever invokes it (no `soffice`, no `libreoffice --headless` call in test.sh or entrypoint.sh). If it's intended as a tool the agent can use to convert deliverables, that intent must be in a comment. Otherwise pure bloat.
- Q5 (labels accurate): ✗ no `dock.benchmark.data_revision` label; the parquet at L29 comes from `refs/convert/parquet` which is mutable. Dataset isn't pinned.
- Q6 (entrypoint sane): ⚠ L94 uses `python3 -c` with the environment variable `$DOCK_TASK_ID` interpolated into the Python source via shell — any task id containing a quote would break it. Minor, but safer to `os.environ.get('DOCK_TASK_ID')`.
- Q7 (subtle smells): ⚠ `HF_USERNAME` and `HF_TOKEN` referenced but never documented in labels; if they're empty the upload is silently skipped with only a stderr line. That's OK for externally graded mode but deserves an explicit LABEL or a comment.
- Verdict: broken (Q5 no: missing `data_revision` for a dataset that exists + Q4 unjustified bloat). The image produces results the labels don't accurately describe.

### agents/claude-code/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓ ubuntu → apt curl → nodesource → nodejs → npm install agent → pack install.sh + entrypoint.sh
- Q2 (comments): ✓ each block commented; the combination-template rationale for /opt/agent is stated
- Q3 (dead code): ✓
- Q4 (bloat): ⚠ the combination-template `/opt/agent/install.sh` duplicates the same nodesource+npm logic already run earlier in the Dockerfile. That's the combination-template design, not a bug, but it means the image carries the install.sh forever as dead code *for the agent's own use* — only consumed downstream. Worth a comment.
- Q5 (labels accurate): ✓ `dock.agent.version="2.1.104"` matches `npm install -g @anthropic-ai/claude-code@2.1.104` on L23 exactly. Green.
- Q6 (entrypoint sane): ✓ defaults ANTHROPIC_BASE_URL and API_KEY, disables experimental betas, uses `-p` print mode with `--dangerously-skip-permissions`
- Q7 (subtle smells): ⚠ `sk-proxy` is the placeholder `ANTHROPIC_API_KEY` default on L46 — the allowlist in dockerfile_inspection.rs covers `sk-proxy`, good. Separate minor: L15 and L19 both do `apt-get install -y` in different RUNs; L19 adds nodejs after running the nodesource setup script. Both cleanups are on their own lines — OK per RULES.md 10(b).
- Verdict: healthy

### agents/bob/Dockerfile

- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓ ubuntu → curl → node → bob tarball → combination template
- Q2 (comments): ✓ Excellent — L23–27 explains WHY the upstream install script is bypassed (mutable S3 version file). Best comment in the sample.
- Q3 (dead code): ✓ `ARG BOB_VERSION=1.0.1` is actively consumed on L28 and in install.sh
- Q4 (bloat): ✓ only Node + bob tarball
- Q5 (labels accurate): ✓ `dock.agent.version="1.0.1"` matches `BOB_VERSION=1.0.1` on L14 and `bobshell-${BOB_VERSION}.tgz` on L28
- Q6 (entrypoint sane): ✓ sets OPENAI_BASE_URL, accepts license, disables telemetry, uses `--yolo`
- Q7 (subtle smells): ⚠ if the user bumps `dock.agent.version` label on L11 but forgets to bump `BOB_VERSION` on L14, the label drifts silently. Two-place version coupling is a known smell. Consider deriving the label from the ARG: `LABEL dock.agent.version="${BOB_VERSION}"` after the ARG declaration.
- Verdict: healthy

## Summary

- ✓ healthy: 6 (appworld, aime, humaneval, gdpval→actually broken, claude-code, bob) — adjusted: 5
- ⚠ needs attention: 6 (cybench, mle-bench, swe-bench, swe-bench-pro, swe-lancer, ifeval)
- ✗ broken: 1 (gdpval)

Final counts: healthy=5, needs attention=6, broken=1.

## Top 5 findings

- `benchmarks/gdpval/Dockerfile` L18, L29: ships `libreoffice` (~1 GB) with no code path that uses it, AND omits `dock.benchmark.data_revision` while pulling from the mutable `refs/convert/parquet` ref. Either document the libreoffice usage or drop it; add a pinned parquet commit SHA.
- `benchmarks/cybench/Dockerfile` L39–40 (and `mle-bench` L41, `swe-lancer` L38/L40): upstream-metadata fallback writes the literal string `"TODO"` into `problem.txt`/`answer.txt`. Images silently ship and grade against placeholder data. Should `exit 1` at build time if the expected fields are missing.
- `benchmarks/swe-bench/Dockerfile` L25 and `swe-bench-pro` L27: `pip install ... 2>/dev/null || pip3 install ...` swallows stderr and the fallback silently hides a failed grader install. Downstream grade.py ImportErrors become reward=0 instead of a build failure.
- `benchmarks/ifeval/Dockerfile` L111–157: defines `test_instruction_following_loose()` inside a heredoc but never calls it (L166 only invokes the strict variant). Dead helper; either use it for loose scoring or delete it.
- `benchmarks/swe-lancer/Dockerfile` L40: installs a `run_tests.sh` stub that prints "TODO" and exits 1, which test.sh interprets as reward=0. Missing upstream should produce a hard build failure, not a silent zero-score.

## Rule catalog gap analysis

The walk surfaced several patterns the mechanical `dockerfile_inspection.rs` rules could catch:

- new rule `todo_string_literal`: flag `"TODO"` or `echo "TODO` inside a `RUN` line, not just inside `#` comments. Covers the cybench/mle-bench/swe-lancer data-fallback smell (the existing TODO-in-comment rule already handles the comment case).
- new rule `silent_pip_fallback`: flag `pip install ... 2>/dev/null || pip` or any `pip install ... || true` pattern. Swallowing stderr on a grader install is a latent data-quality bug.
- new rule `duplicate_arg_after_from`: flag a second `ARG DOCK_TASK_ID` (or any ARG whose same name already appeared before `FROM` in the same stage). The Docker requirement for pre-FROM ARG doesn't demand the ARG be re-declared post-FROM when only an `ENV` is needed — the ENV alone is enough.
- new rule `phantom_pip_uninstall`: flag `pip uninstall -y <pkg>` in a different RUN layer from its matching `pip install <pkg>`. The uninstall doesn't reclaim space and misleads readers. Require them to be in the same RUN (or at least the same layer).
- new rule `version_label_arg_mismatch`: if a Dockerfile declares `ARG FOO_VERSION=X` and a `LABEL ... version="Y"`, require X == Y. Catches the bob.ibm drift risk.
- new rule `missing_data_revision_when_fetching_mutable_ref`: if a RUN pulls from `refs/convert/parquet`, `main`, `master`, or `HEAD` and there is no `dock.benchmark.data_revision` label pinning a commit SHA, fail the image. Catches the gdpval gap.
- new rule `install_order_pip_before_apt`: warn when `pip install` comes before `apt-get install` in the same Dockerfile — apt layer churn invalidates pip cache on every rebuild. Covers swe-bench L25 vs L29.
- new rule `unreferenced_heavy_package`: warn when a known-large apt package (`libreoffice`, `texlive-full`, `cuda-toolkit`) is installed but not referenced by any RUN, entrypoint, or test script in the same directory. Catches gdpval libreoffice.
