---
benchmark: hwe-bench
host: local docker (macOS, linux/amd64 emulation)
commit: add-hwe-bench
---
# Audit — hwe-bench (pku-liang/hwe-bench, 5 projects / 169 cases)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | per-task PULL + overlay (rule 24g), not a source build: `build.sh` derives the base image ref + repo path from the task id (pure string transform, no dataset lookup), then `docker build --platform linux/amd64` an overlay `FROM ghcr.io/pku-liang/<org>_m_<repo>:pr-<number>` — verified on one case per project. All 5 base-image slugs anonymously pullable from GHCR (HTTP 200) |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | gold not baked into agent space (ships in `/tasks/0/config.json`, root-only `600`; `solution.sh` applies it as root); `/tests` root-only (`700`, `root:root`) holds the grader + the case's self-contained `tb_script`; the task id is excluded from the agent env (framework `env -i`, rule 7); the design repo at `/home/<repo>` is `chown`ed to `agent` |
| oracle | ✓ | gold=1.0 / no-op=0.0 confirmed on **one representative per project — all five** (`eval-containers oracle hwe-bench --task-id <id> --local --platform linux/amd64`): ibex `lowrisc__ibex-2232`, cva6 `openhwgroup__cva6-1482`, caliptra `chipsalliance__caliptra-rtl-963`, rocket-chip `chipsalliance__rocket-chip-3526`, XiangShan `openxiangshan__xiangshan-39`. Gold = the dataset's own `fix_patch`, graded by HWE's own simulation method. ibex independently confirmed at RTL level: buggy base → `TEST: pmp_debug_dm_access ... FAIL`; `+fix_patch` → `... PASS`. All five ids are registered in the oracle `SPECIAL` gate. (Chisel repos rocket-chip/XiangShan elaborate Scala→Verilog via Mill inside `tb_script` before Verilator sim; cva6 uses Spike — all emit the same markers, grader unchanged.) Task ids are **lowercase** (Docker-safe: the id is the per-task image namespace and must survive `docker compose`'s raw `${EVAL_TASK_ID}` interpolation, which YAML can't lowercase); `build.sh` maps the two mixed-case orgs (`lowRISC`, `OpenXiangShan`) back to their HF JSONL filename at build time (`HWE_HF_SLUG`). |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published leaderboard score |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner (`compose/services.yaml`; runner on the `internal: true` network only); `LABEL eval.benchmark.internet="false"` — grading is offline (Verilator baked into the base image; no dataset package at run time; `tb_script` is self-contained) |
| agent-nonroot | ✓ | agent runs via the shared runner (`core/runner/process-compose.yaml`) as `gosu agent`; `entrypoint.sh` `chown -R agent:agent /home/<repo>` hands the repo to the agent; the benchmark image adds no agent/root override |
| secrets-isolated | ✓ | no secrets in `Dockerfile`/`build.sh` (no `ENV`/`COPY` of credentials; `build.sh`/Dockerfile only pull the public dataset + base image); model creds enter via the framework gateway (rule 8a) |
| resource-limited | ? | CPU/memory caps not audited (shared runner default `cpus: 2` / `2G`); Verilator compile of the ibex core is heavier than a pytest — measure peak and set a preset if the default is tight |

## Size

| Metric | Value |
|--------|-------|
| base image | per-task; the published `ghcr.io/pku-liang/<org>_m_<repo>:pr-<number>` (design repo at `/home/<repo>` + git + sim toolchain baked in). ibex base ≈ **1.42 GB** uncompressed; the Chisel repos (rocket-chip, XiangShan) carry a Mill/Scala toolchain and are larger |
| per-task image | base + eval overlay (apt `curl python3` + task metadata + `tb_script` + grader); one image per case. Cases in the same project share that project's `:base` layers (35/35/16/31/52 across the 5 projects) |

## Speed

| Metric | Value |
|--------|-------|
| build | per-task PULL of the published base + overlay (apt `curl python3`, fetch the JSONL row, bake); overlay itself ~1 min once the base is cached |
| grade | HWE's own simulation method: reset to base, apply the candidate diff, run `tb_script` (per project: Verilator build+sim; Spike for cva6; Mill Chisel-elaboration → Verilator for the Chisel repos), parse `TEST:` markers; resolved iff every fail-to-pass test PASSES |
| end-to-end | Verilator build+sim ≈ **2–4 min per grade** under amd64 emulation on Apple Silicon (native x86 faster); the Chisel repos are heavier (Mill elaboration adds minutes); oracle (gold + no-op) ≈ 2× that plus the base pull |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (no live agent run yet) |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| released | ✗ | `released` label deferred (rule 21a) — needs a recorded replay fixture from a live sweep (M3) |
| pull size | — | not published (per-task; base pulled from GHCR + overlay) |

## Coverage / status

All five source-buildable projects (169 gradeable cases) ship together, one grader
for all. OpenTitan (245 cases, 59% of the full 417) is permanently dropped —
commercial Synopsys VCS, images not distributed. Of the 172 open-tooling cases, 3
are excluded because their gold fix lives entirely or partly in a git submodule
(`Subproject commit` pointer bump), which the grader cannot score under the
contract that the agent edits only tracked superproject HDL source:
`chipsalliance__rocket-chip-177` (fix is *only* a submodule bump — unsolvable), and
`openxiangshan__xiangshan-2246` / `openxiangshan__xiangshan-2781` (mixed
submodule + source, where the source hunk alone does not pass the failing test —
verified: gold scores 0 under the fixed grader). The remaining 11 mixed
submodule+source cases are kept (source hunk alone resolves the test, gold=1); both
`grade.sh` and `solution.sh` ignore submodule-pointer hunks. Net: 169.

- **Shipped, unreleased-but-runnable:** ibex (35), cva6 (35), caliptra-rtl (16),
  rocket-chip (31), XiangShan (52). One representative per project is oracle-proven
  and registered in the oracle `SPECIAL` gate.
- **Release (deferred, rule 21a):** record a replay fixture from a live agent sweep,
  then set `LABEL eval.benchmark.released="true"`.
