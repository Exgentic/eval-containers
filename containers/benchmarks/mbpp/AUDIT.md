---
benchmark: mbpp
host: local podman+Rosetta
commit: 1a344fd
---
# Audit — mbpp

`✓` verified (a check passed) · `◐` partial (holds in one surface, open in another) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on Rosetta (podman, no QEMU) |
| running | ? | not run with a live agent |
| isolation | ? | not audited per-benchmark |
| reward-hacking | ✓ | grader decides reward, not the candidate's exit code: shared `/eval-grade` harness — reward seeded to fail, set to 1.0 only on a genuine pass (this PR). Proven: `exit()`/`sys.exit`/`os._exit` cheat → 0.0; correct → 1.0. Scope: trivial/accidental self-passes only — a deliberate root `open(reward,'w')` forge stays open (see Notes) |
| oracle | ✓ | gold=1.0 / no-op=0.0 — `eval-containers oracle mbpp` |
| traces-reviewed | ? | |
| replicate-official | ? | |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ◐ | compose: runner on `internal: true` (no egress) ✓. single-image: rule-24d `iptables --uid-owner` on the agent uid not verified here |
| agent-nonroot | ✓ | agent runs as uid 1002 via `gosu agent` (`core/process-compose/process-compose.yaml`) |
| secrets-isolated | ◐ | compose: real key only in the separate `gateway` service; agent gets `sk-proxy` ✓. single-image: verifier runs `/grade.sh` (and the candidate it execs) **as root** — see reward-forge residual |
| resource-limited | ✓ | compose `deploy.resources.limits` 2 cpu / 2 GB (shared `compose/services.yaml`) |

## Size

| Metric | Value |
|--------|-------|
| image | ? |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? |
| grade | ? |
| end-to-end | ? |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| pull size | — | not published |

## Notes

**Reward-hacking fix (shared `/eval-grade`).** The prior `/grade.sh` ran the
agent's stdout + tests as one program and scored `reward = (exit code == 0)`, so
`exit()`/`sys.exit()`/`os._exit()`/a crash scored 1.0 without solving the task
(rule 5). It now pipes the program into the shared `/eval-grade` harness
(`benchmarks/RULES.md` 22, `exec` mode): the reward is seeded to fail and set to
1.0 only if the program runs to completion with no exception — the exit code is
never read. **Oracle re-run (this image, tasks 0/1): gold = 1.0, no-op = 0.0.**

**Residuals (open):**
- **Reward forge — verifier runs as root.** `/grade.sh` runs `/eval-grade` (which
  `exec`s the candidate) as root, so a deliberate completion can write the reward
  path itself — `open('/logs/verifier/reward.txt','w').write('1.0')` — and forge a
  pass. The trivial/accidental self-passes are closed; this is not, and closes
  only by dropping privilege (non-root, no-network uid; reward owned by the
  verifier). Shared across the code benchmarks; full write-up in
  `bigcodebench/AUDIT.md`.
- **`exec` mode treats any `SystemExit` as failure** — harmless here (assert-based
  tests), but a test that self-exits 0 on success would false-negative.
