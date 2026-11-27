---
benchmark: audiomc
host: —
commit: (pending)
---
# Audit — audiomc

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ? | not yet built |
| running | ? | no run yet; needs an audio-input model (Gemini) |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | there is no derivable gold: the reward is a rubric judge's verdict on a free-form spoken-dialogue reply, and the dataset ships no reference answers |
| traces-reviewed | ? | |
| replicate-official | ? | the judge is the run's pinned model, where upstream uses o4-mini, so scores are not directly comparable with the paper's |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| — | — | — | — | — | — | no run recorded yet |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | runner on `internal` network only; the only outbound is the model call to the gateway |
| agent-nonroot | n/a | no agent — the native harness runs privileged in place of one (rule 12c) |
| secrets-isolated | ? | real key on gateway; runner holds `sk-proxy` placeholder |
| resource-limited | ? | compose runner limits: 1 CPU / 2G |

## Size

| Metric | Value |
|--------|-------|
| image | ? (~5.8 GB expected: 5.7 GB of 48 kHz wav) |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? (one 5.3 GB download) |
| grade | ? (one judge call per rubric, 3.8 on average) |
| end-to-end | ? (one model request per task) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (one audio completion + ~3.8 text judge calls) |
| full suite | ? (452 audio completions + 1,712 judge calls) |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ? | not yet pushed |
| released | ✗ | no replay fixture; `eval.benchmark.released` label not set |
