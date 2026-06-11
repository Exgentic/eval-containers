# Per-task benchmark: one eval image per task — evals/<benchmark>-<task>--<agent>.
# EVAL_TASK_ID (the ARG below, consumed by FROM) selects which task's image this
# single-mode pin resolves to. (benchmarks/RULES.md — eval-image naming.)
ARG EVAL_TASK_ID
FROM ghcr.io/exgentic/evals/terminal-bench-${EVAL_TASK_ID}--claude-code:latest
