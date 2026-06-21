---
benchmark: bigcodebench
host: local podman+Rosetta
commit: 1a344fd
---
# Audit — bigcodebench

`✓` verified (a check passed) · `◐` partial (holds in one surface, open in another) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on Rosetta (podman, no QEMU) |
| running | ? | not run with a live agent |
| isolation | ✓ | tests root-only `0600` (`all.jsonl` + per-task dir); agent runs as uid 1002 (`gosu agent`, `env -i`, no `EVAL_TASK_ID`) — cannot read the suite |
| reward-hacking | ✓ | grader decides reward, not the candidate's exit code: shared `/eval-grade` harness — reward seeded to fail, set to 1.0 only on a genuine pass; success read from TestResult, not the patchable runner. Proven: `exit()`/`sys.exit`/`os._exit`/zero-test/runner-monkeypatch → 0.0; correct → 1.0. Scope: trivial/accidental self-passes only — a deliberate root `open(reward,'w')` forge stays open (verifier-runs-as-root residual) |
| oracle | ✓ | gold=1.0 / no-op=0.0 — `eval-containers oracle bigcodebench` |
| traces-reviewed | ? | |
| replicate-official | ? | |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ◐ | compose: runner on `internal: true` (no egress) ✓. single-image: rule-24d `iptables --uid-owner` on agent uid not verified here — until then a single-image agent could fetch the public `canonical_solution` from HF |
| agent-nonroot | ✓ | agent runs as uid 1002 via `gosu agent` (`core/runner/process-compose.yaml`) |
| secrets-isolated | ◐ | compose: real key only in the separate `gateway` service; agent gets `sk-proxy` ✓. single-image: verifier runs `/grade.sh` **as root** in the same container as the gateway key — candidate code executed by the grader (see reward-hacking residual) runs as root; sandbox it before trusting single-image safety |
| resource-limited | ✓ | compose `deploy.resources.limits` 2 cpu / 2 GB |

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

**Reward-hacking fix (grader no longer trusts the candidate's exit code).** The
prior `/grade.sh` concatenated the agent's stdout ahead of the test and scored
`reward = (python exit code == 0)`. Because the agent's code ran first in the
same interpreter, `import sys; sys.exit(0)` (or `exit()`/`os._exit(0)`/a runner
monkeypatch) scored 1.0 without solving the task — violating rule 5 (the agent
must not be able to forge the test outcome). The grader now follows the
documented HumanEval/BigCodeBench method via the shared `/eval-grade` harness
(`benchmarks/RULES.md` 22): `grade.sh` pipes the program into `/eval-grade` with
`exit()`/`quit()` neutralized; the reward file is seeded to fail and overwritten
with 1.0 only on a genuine unittest pass — the exit code is never read. (The
guard is deliberately minimal — host-damage isolation is the container's job, not
the grader's, so BigCodeBench's legitimate tool-call tasks still run.)

**Oracle re-run — passes (all five).** The grader change was re-validated
end-to-end in the real benchmark images: the new `/grade.sh` + `/eval-grade`
overlaid on `ghcr.io/exgentic/benchmarks/<bench>:latest` and run via the actual
oracle runner (real `solution.sh`, `/entrypoint.sh`, materialize, as root).
**gold = 1.0, no-op = 0.0** for bigcodebench (tasks 0, 2) and humaneval /
humanevalplus / mbpp / mbppplus (tasks 0, 1) — stdlib tasks, so libs aren't a
variable. A separate 50-row real-data check found the new grader **agrees with
the old grader on every task** (no regression) and rejects every no-op. Still for
CI: a from-source rebuild of `benchmark-base-hf` (the overlay places `/eval-grade`
at the same path the base Dockerfile does, but doesn't re-run that build), the
replay sweep, and a broader task sweep (lib-heavy tasks depend on image runtime
deps — a pre-existing question, unaffected by this change).

**Residuals (open):**
- **Verifier runs as root → a deliberate completion can forge the reward.**
  `/grade.sh` runs `/eval-grade` (which `exec`s the candidate) as root, so the
  completion can write the reward path itself — `open('/logs/verifier/reward.txt',
  'w').write('1.0')` (demonstrated) — and score 1.0 regardless of the tests. The
  reward-hacking row above covers the *trivial/accidental* self-passes (exit /
  quit / crash / runner-patch), which ARE closed; this *deliberate* forge is the
  same class as the same-interpreter monkeypatch and is NOT. Both close only by
  dropping privilege: run the candidate as a dedicated non-root, no-network uid
  with `reward.txt` owned by the verifier uid (as the cited inspect_evals impl
  does with a disposable sandbox), so the candidate's `open(...,'w')` gets EACCES.
  No cheap in-process mitigation exists (the reward path is public per rule 18, so
  hiding it is theater; a subprocess stops the monkeypatch but not the file
  write). In single-image mode that container also holds the real API key + egress,
  raising the stakes.
- **Single-image egress** (`egress-blocked = ◐`): verify the rule-24d
  `iptables --uid-owner` rule actually blocks the agent uid; the public dataset
  contains `canonical_solution`.
- **exec-mode treats any `SystemExit` as failure** (incl. a passing test that
  ends in `unittest.main()`/`exit(0)`). Harmless for HumanEval/MBPP/EvalPlus
  (assert/`check`-based), but a benchmark whose test self-exits 0 on success
  would false-negative under `exec` mode — use `unittest` mode there.

**Systemic.** The same exit-code-trust pattern was fixed in lockstep in
`humaneval`, `humanevalplus`, `mbpp`, and `mbppplus` (this PR). Count-based code
graders (`scicode`, `code-contests`) use a different scorer and are not touched
here.
