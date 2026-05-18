# commonsenseqa

CommonsenseQA - commonsense multiple choice QA

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1221 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://www.tau-nlp.sites.tau.ac.il/commonsenseqa](https://www.tau-nlp.sites.tau.ac.il/commonsenseqa) |
| Paper | [paper](https://arxiv.org/abs/1811.00937) |
| Dataset revision | `94630fe30dad47192a8546eb75f094926d47e155` |

## What the agent sees

The agent receives a task of the form: "Answer this commonsense multiple choice question. Print only the letter of the correct answer (A, B, C, D, or E), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run commonsenseqa`
- `README.md` — this file
