"""Video-MME-v2 native harness: upstream's default protocol (64 frames, no
subtitles, no reasoning prompt), one model call per question.

Each call carries the 64 frames as images, then upstream's WO_SUB_PROMPT, then
`Question: <question>\\n<options>\\n<INSTRUCT_PROMPT>` (build_chat_messages in
evaluation/test_video_mme_v2.py). The 4 questions are asked independently, as
upstream does. Prints one JSON line per question, {question_id, prediction},
for /grade.sh; a failed call predicts '' as upstream's does, and the harness
then exits non-zero so the run records the failure.
"""

import base64
import json
import os
import sys
import urllib.request

WO_SUB_PROMPT = "These are the frames of a video."
INSTRUCT_PROMPT = (
    "Select the best answer to the following multiple-choice question based on the video. "
    "Respond with only the letter (A, B, C, D, E, F, G, or H) of the correct option."
)
CALL_TIMEOUT = 300  # seconds per call; 4 calls fit the 1800 s run bound


def chat(content):
    payload = {"model": os.environ.get("MODEL", ""), "messages": [{"role": "user", "content": content}]}
    req = urllib.request.Request(
        os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1").rstrip("/") + "/chat/completions",
        data=json.dumps(payload).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
    )
    with urllib.request.urlopen(req, timeout=CALL_TIMEOUT) as resp:
        return json.loads(resp.read())["choices"][0]["message"].get("content") or ""


def main():
    frames = []
    for name in sorted(os.listdir("/tasks/frames")):
        with open(f"/tasks/frames/{name}", "rb") as f:
            url = "data:image/jpeg;base64," + base64.b64encode(f.read()).decode()
        frames.append({"type": "image_url", "image_url": {"url": url}})

    failed = 0
    for line in open("/tasks/questions.jsonl"):
        row = json.loads(line)
        content = frames + [
            {"type": "text", "text": WO_SUB_PROMPT},
            {"type": "text", "text": f"Question: {row['question']}\n{row['options']}\n{INSTRUCT_PROMPT}"},
        ]
        try:
            reply = chat(content).strip()
        except Exception as e:  # noqa: BLE001 — upstream scores a failed call as ''
            print(f"[video-mme-v2] {row['question_id']}: {e}", file=sys.stderr)
            reply, failed = "", failed + 1
        print(json.dumps({"question_id": row["question_id"], "prediction": reply}), flush=True)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
