# Dockerfile Health Inspection

**Status:** Draft
**Date:** April 2026

## Abstract

Structural validation (files exist, labels present) tells you a
Dockerfile is **present**. `docker build` tells you it **compiles**.
Neither tells you the Dockerfile is **sane** — that it has no
hardcoded secrets, no version drift, no convention violations, no
TODOs left in, and no subtle mistakes a reviewer would flag on sight.

Dockerfile health inspection closes that gap. For every benchmark and
agent Dockerfile, we check it against a catalog of green, yellow, and
red signals.

This document defines what we look for, how we classify signals, how
the mechanical checks work, and what a manual audit covers.

## The inspection unit

One Dockerfile is one inspection record:

- **Input:** the raw text of a single `Dockerfile` under
  `benchmarks/<name>/Dockerfile` or `agents/<name>/Dockerfile`.
- **Context:** the directory name (for label consistency checks),
  the directory's expected type (benchmark or agent), any sibling
  files in the same directory.
- **Output:** a health verdict (`green` | `yellow` | `red`) with the
  matching signals listed.

## Signal catalog

### Red signals (any one triggers a `red` verdict)

- **Hardcoded secret.** Any line contains a literal API key, token,
  or password. Patterns to flag: `sk-[A-Za-z0-9]{20,}`, `ghp_[A-Za-z0-9]{20,}`,
  `AKIA[0-9A-Z]{16}`, `xoxb-[0-9]+-`, or any `ENV <key>=` where the
  key name contains `KEY`, `TOKEN`, `PASSWORD`, `SECRET` and the value
  is not empty, `""`, `unset`, or a placeholder like `sk-proxy`.
- **Unpinned pip install.** `pip install foo` without `==X.Y.Z`.
  Allowed: `pip install -r requirements.txt` where the requirements
  file itself is pinned.
- **Unpinned npm install.** `npm install -g foo` without `@X.Y.Z`.
- **Unpinned curl install.** `curl ... | bash` or `curl ... | sh`
  where the URL contains no version tag and the script source isn't
  pinned by a release directory. `deb.nodesource.com/setup_22.x` is
  allowed (major version is pinned).
- **Legacy env var.** Any `$TASK_ID`, `${TASK_ID}`, `$BENCHMARK`,
  `${BENCHMARK}` reference — must be `$EVAL_TASK_ID` / `$EVAL_BENCHMARK`
  after the April 2026 migration.
- **Untagged FROM.** `FROM ubuntu` with no `:tag`. Pulls whatever is
  tagged `latest` at build time. Never reproducible.
- **Label drift.** The `eval.benchmark.name` or `eval.agent.name` label
  does not match the directory name.
- **Missing required label.** See `benchmarks/RULES.md` rule 21 and
  `agents/RULES.md` rule 14 for what's required per image type.
- **TODO / FIXME in production.** A comment contains `TODO`, `FIXME`,
  or `XXX` as a standalone token. Exception: a comment inside a
  "future work" block at the bottom of the file, explicitly marked
  `# FUTURE:`.
- **Apt install without cleanup.** `apt-get install` in a `RUN` that
  doesn't also `rm -rf /var/lib/apt/lists/*` on the same layer. The
  cleanup in a separate `RUN` does nothing. See RULES.md rule 10(b).
- **Pip install with cache.** `pip install` without `--no-cache-dir`.
  See RULES.md rule 10(b).
- **ENV leaks a cache directory.** `COPY ~/.cache`, `COPY ~/.npm`,
  `COPY ~/.cargo/registry`. See RULES.md rule 10(c).

### Yellow signals (warn but don't fail)

- **Non-slim base when slim is available.** `FROM python:3.12` where
  `python:3.12-slim` would work. Might be intentional (header files
  needed for compilation) but worth a look.
- **`eval.benchmark.data_revision` empty or `latest`.** The label is
  present but its value is empty, `latest`, `main`, `master`, or `HEAD`.
  Benchmarks should pin a specific commit / dataset revision.
- **Very long `RUN`.** A single `RUN` with more than ~20 commands
  chained by `&&`. Readable top-to-bottom is an RULES.md 10(e) goal.
- **Missing `WORKDIR`.** The Dockerfile uses `cd` inside `RUN` steps
  instead of `WORKDIR`. Not a bug, but a smell.
- **Very long Dockerfile.** More than 150 lines. Might be doing too
  much; consider whether the benchmark/agent has unique needs or
  whether common pieces should move to a shared base image.

### Green signals (all must be present for a `green` verdict)

- **Reproducible.** No unpinned installs, no untagged FROM, no mutable
  remote script sources.
- **Labeled.** All required `eval-containers.*` labels present and consistent
  with the directory name.
- **Hygienic.** Follows RULES.md principle 10: slim bases, in-layer
  cleanup, no caches in layers.
- **Uses EVAL-prefixed env vars.** No legacy `TASK_ID` or `BENCHMARK`
  references.
- **No secrets.** No keys, tokens, or passwords in the image.

## Classification rules

```
if any red signal:     verdict = red
elif any yellow:       verdict = yellow
else:                  verdict = green
```

Red is a CI failure. Yellow is a warning. Green is healthy.

## Layered checking

Parallel structure to [TRAJECTORY.md](TRAJECTORY.md).

**Layer 1 — mechanical rules.** Automated by `tests/dockerfile_inspection.rs`.
Every red and yellow signal above is a regex, substring, or line-count
check. Runs in seconds across every Dockerfile. Rule IDs match the
signal names in this doc so the spec and the engine can't drift.

**Layer 2 — procedural audit.** The checklist below. A human, an AI
assistant, or a script walks the Dockerfiles and applies judgment to
the things mechanical rules can't catch: is the install order
reasonable? Does the image do too much? Is the comment quality high
enough that the next maintainer can understand why?

**Layer 3 — external linter.** [`hadolint`](https://github.com/hadolint/hadolint)
is a standard Dockerfile linter that catches ~50 common smells
(DL3000–DL4006 rule set). If `hadolint` is installed locally,
`tests/dockerfile_inspection.rs` optionally runs it and merges
findings. No hard dependency — the test passes without it.

## Audit procedure

Run this when you want a judgment-level review of Dockerfile health.
Applies to a single Dockerfile, a batch, or the whole fleet.

### Scope

- **Input:** one or more `Dockerfile` paths under `benchmarks/*/` or
  `agents/*/`.
- **Context:** the parent directory name, its sibling files
  (`compose.yaml`, `install.sh`, `entrypoint.sh` if present), the
  directory's `RULES.md` in its parent directory.

### Steps

1. **Run mechanical rules first.** `cargo test --test dockerfile_inspection
   -- --ignored`. Note what it found. The audit's job is to find what
   rules missed, not to duplicate them.

2. **Read the Dockerfile end to end.** Ask the seven questions below,
   one per Dockerfile. Mark each yes / no / n.a. with a one-line reason.

   | # | Question |
   |---|---|
   | 1 | Does the install sequence make sense? (base → system deps → language runtime → app → entrypoint) |
   | 2 | Are comments sufficient — could a new maintainer understand WHY each layer exists? |
   | 3 | Is there dead code (unused ARGs, dangling COPY destinations, commented-out blocks)? |
   | 4 | Does the image include anything the runtime doesn't need (build toolchains, docs, sample data)? |
   | 5 | Does the label set correctly describe the image content? (agent version matches install command, dataset revision matches fetch URL, etc.) |
   | 6 | Is the entrypoint sane — reads the right env vars, handles missing defaults, exits cleanly? |
   | 7 | Any subtle smells a reviewer would flag but the rules didn't catch? |

3. **Classify.**
   - ✓ **healthy** if all seven answers are yes or n.a.
   - ⚠ **needs attention** if any question is no but the image still works.
   - ✗ **broken** if question 5 or 6 is no — the image is wrong.

### Output format

One markdown report, one entry per Dockerfile:

```
## benchmarks/aime/Dockerfile
- Mechanical rules: ✓ (0 findings)
- Q1 (install order): ✓
- Q2 (comments): ✓
- Q3 (dead code): ✓
- Q4 (bloat): ⚠ ships pyarrow for build-time parquet parse, uninstalled at L23 — good
- Q5 (labels): ✓ data_revision matches URL sha
- Q6 (entrypoint): ✓
- Q7 (smells): ✓
- Verdict: healthy
```

Followed by a summary count and top 3 suggested fixes (if any).

### When to run

- Before cutting a release (whole fleet)
- When `dockerfile_inspection` flags a yellow
- When a new benchmark or agent batch lands
- When RULES.md changes (the old Dockerfiles may drift)
- Quarterly, as a health check

### Who runs it

Anyone. Same principle as [TRAJECTORY.md](TRAJECTORY.md): the
procedure is toolchain-agnostic. A human reads the Dockerfile in
their editor. An AI assistant reads this checklist and executes it.
A script could implement parts of it. The output format is fixed so
findings are comparable.

## References

- [RULES.md](../RULES.md) — principle 9 (pin versions), principle 10 (image hygiene), principle 11 (env var namespace)
- [benchmarks/RULES.md](../benchmarks/RULES.md) — required benchmark labels (rule 21)
- [agents/RULES.md](../agents/RULES.md) — required agent labels (rule 14)
- [TRAJECTORY.md](TRAJECTORY.md) — parallel spec for runtime task health
- [hadolint](https://github.com/hadolint/hadolint) — external Dockerfile linter
