# video-mme-v2

Video-MME-v2 — video understanding: 4 multiple-choice questions (A–H) on one
video, which the model sees as 64 sampled frames, scored with upstream's grouped
non-linear rating. Model-only (native harness), one image per video.

**Status:** Not yet released — no replay fixture landed (see
[`.agents/benchmarks/RULES.md`](../../../.agents/benchmarks/RULES.md) rule 21a).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 800 videos (3200 questions, 4 per video); task id = upstream video id, `001`–`800` (`tasks.txt`) |
| Environment | per-task (one image per video) |
| Agent | `native` (benchmarks rule 12a) |
| Internet required | false |
| Released | no |
| Upstream | [github.com/MME-Benchmarks/Video-MME-v2](https://github.com/MME-Benchmarks/Video-MME-v2) |
| Dataset | [MME-Benchmarks/Video-MME-v2](https://huggingface.co/datasets/MME-Benchmarks/Video-MME-v2) |
| Paper | [arXiv:2604.05015](https://arxiv.org/abs/2604.05015) |
| Dataset revision | `6e4bebb03202e1ddbf3d37703e560e51c5aa2d64` |
| License | Academic research only; no redistribution without the authors' approval |

## Why per-task and native

The 800 videos total ~105 GB (40 zips of 20 videos), so each image bakes one
video: its 4 questions and the frames sampled from it. Upstream evaluates a model,
not an agent: one call per question, with the video given as frames. Asking the
4 questions in one agent session would also leak answers between them (video
001's question 2 offers "Because her mom is Malaysian Chinese", which answers
question 1). So the benchmark ships its own harness and pairs only with
`native`.

Because the frames go as ordinary images, any model that accepts images through
the gateway's OpenAI-compatible `chat/completions` can run it. No video input is
needed.

## How a task runs

Build (`materialize.py`): from the pinned revision, read the video's 4 rows of
`test.parquet`. Read its mp4 out of the zip with HTTP range requests. Sample 64
frames with ffmpeg at upstream's indices (`extract_frames`, `nframe=64`:
`step = n / 65`, frames `int(i * step)` for `i` in 1..64). Only the frames
(~7 MB of JPEG) and the questions ship; the mp4 does not.

Run (`/harness.sh` → `run_video_mme_v2.py`): for each question, one
`chat/completions` call through the gateway. The user message is upstream's
`build_chat_messages` for the default config `Video-MME-v2_64frame`: the 64
frames, then `These are the frames of a video.`, then
`Question: <question>\n<options>\n` and upstream's instruction to answer with
only the letter. Each request is ~9 MB. The harness prints one
`{question_id, prediction}` line per question. A failed call predicts `''`, as
upstream's does, and makes the harness exit non-zero so the run records it.

## How it's graded

`/grade.sh` (`grade.py`, root-only under `/tests`) uses upstream's scoring
unchanged:

- **Answer extraction** (`extract_characters_regex_v2`): strip the answer
  prefixes, then take the first `A`–`H`. A reply longer than 10 words with no
  letter extracts nothing, which counts as wrong.
- **Group rating.** Relevance groups score by how many of the 4 are correct
  (0, 6.25, 25, 56.25, 100). Logic groups score by the leading run of correct
  answers, with upstream's maps for the three structures `[1,2,3,4]`,
  `[1,[2,3],4]` and `[[1,2],3,4]`.

reward = the group's rating / 100. The mean reward over the 800 tasks is
upstream's overall rating (/100). `/output/task/video-mme-v2.json` records each
question's extracted letter and correctness, from which per-question accuracy
can be computed.

## Differences from upstream

- **Decoding.** Upstream decodes greedily (`temperature=0`, 4096 new tokens).
  The harness sends only `model` and `messages`, so the provider's defaults
  apply. Some reasoning models refuse a `temperature`, and the reply is one
  letter.
- **Frames.** Upstream decodes with decord and saves each frame with PIL as
  JPEG (default quality 75). Here ffmpeg selects the same frame numbers and
  saves them at `-q:v 2`, full resolution (1080p); the provider resizes them,
  not upstream's processor. `n` is the stream's `nb_frames`, which is what
  decord's `len()` reads.
- **Only the default config.** No subtitle modes, no reasoning prompt, no
  1-fps sampling.

## Oracle

`solution.sh` prints each question's dataset answer in the harness's line
format, so gold scores `1.0`; a no-op predicts nothing and scores `0.0`.

## Files

- `Dockerfile` — builder stage (`core/benchmark-base-hf` + ffmpeg) runs `materialize.py`; the final image is `core/benchmark-base-python-slim` with `/tasks`, the harness and the grader.
- `materialize.py` — bakes one video's questions and frames into `/tasks`.
- `run_video_mme_v2.py` — the native harness.
- `grade.py` — upstream's extraction and group rating.
- `solution.sh` — oracle gold (mounted, never baked).
- `tasks.txt` — the 800 task ids.
- `compose.yaml` — compose surface (`runner-native`); k8s uses the shared chart (`presets/video-mme-v2.yaml` pins `agent: native`).

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`): `BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running

```bash
eval-containers build bench video-mme-v2 --task-id 001 --local
eval-containers oracle video-mme-v2 --task-id 001 --local
eval-containers run video-mme-v2 --agent native --model <image-capable model> --task-id 001 --local --mode compose
```
