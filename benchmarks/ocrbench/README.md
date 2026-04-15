# ocrbench

OCRBench - benchmark for OCR capability of large multimodal models

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/Yuliang-Liu/MultimodalOCR](https://github.com/Yuliang-Liu/MultimodalOCR) |
| Paper | [paper](https://arxiv.org/abs/2305.07895) |
| Dataset revision | `92a54bd1384387c178d5a07140a2d85e0a3d12e1` |

## What the agent sees

The agent receives a task of the form: "Look at the provided image and answer the following question about its text content. Print only the answer (the requested text or short phrase), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run ocrbench`
- `README.md` — this file
