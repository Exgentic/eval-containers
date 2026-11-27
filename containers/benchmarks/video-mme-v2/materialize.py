"""Bake one Video-MME-v2 video as a task: its 4 questions and 64 frames.

    /tasks/id.txt             upstream video id (e.g. 001)
    /tasks/questions.jsonl    the video's 4 rows of test.parquet, upstream order
    /tasks/frames/NN.jpg      the 64 frames upstream's default config samples

The videos ship as 40 zips of 20 (videos/001.zip holds 001-020.mp4, 1-5 GB
each), so the one mp4 is read out of its zip with HTTP range requests instead
of downloading the archive. Frame indices are upstream's extract_frames()
(evaluation/test_video_mme_v2.py, nframe=64): step = n / 65, frames
int(i * step) for i in 1..64, n being the container's frame count (decord's
len(vid)).
"""

import io
import json
import os
import shutil
import subprocess
import sys
import urllib.request
import zipfile

import pyarrow.parquet as pq

rev, video_id, out = sys.argv[1], sys.argv[2], sys.argv[3]
BASE = f"https://huggingface.co/datasets/MME-Benchmarks/Video-MME-v2/resolve/{rev}"
NFRAME = 64


class HttpFile(io.RawIOBase):
    """A seekable read-only file over HTTP range requests."""

    def __init__(self, url):
        with urllib.request.urlopen(urllib.request.Request(url, method="HEAD")) as r:
            self.url, self.size = r.url, int(r.headers["Content-Length"])
        self.pos = 0

    def readable(self):
        return True

    def seekable(self):
        return True

    def tell(self):
        return self.pos

    def seek(self, off, whence=0):
        self.pos = {0: off, 1: self.pos + off, 2: self.size + off}[whence]
        return self.pos

    def readinto(self, b):
        n = min(len(b), self.size - self.pos)
        if n <= 0:
            return 0
        rng = {"Range": f"bytes={self.pos}-{self.pos + n - 1}"}
        with urllib.request.urlopen(urllib.request.Request(self.url, headers=rng)) as r:
            data = r.read()
        b[: len(data)] = data
        self.pos += len(data)
        return len(data)


urllib.request.urlretrieve(f"{BASE}/test.parquet", "/tmp/test.parquet")
rows = [
    r
    for r in pq.read_table("/tmp/test.parquet").to_pylist()
    if r["video_id"] == video_id
]
assert len(rows) == 4, f"video {video_id!r}: expected 4 questions, got {len(rows)}"

os.makedirs(f"{out}/frames", exist_ok=True)
with open(f"{out}/id.txt", "w") as f:
    f.write(video_id)
with open(f"{out}/questions.jsonl", "w") as f:
    for r in rows:
        f.write(json.dumps(r) + "\n")

archive = f"{BASE}/videos/{(int(video_id) - 1) // 20 + 1:03d}.zip"
with zipfile.ZipFile(io.BufferedReader(HttpFile(archive), 8 << 20)) as z:
    with z.open(f"{video_id}.mp4") as src, open("/tmp/video.mp4", "wb") as dst:
        shutil.copyfileobj(src, dst, 8 << 20)

n = int(
    subprocess.check_output(
        [
            "ffprobe",
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=nb_frames",
            "-of",
            "csv=p=0",
            "/tmp/video.mp4",
        ],
        text=True,
    ).strip()
)
step = n / (NFRAME + 1)
select = "+".join(f"eq(n\\,{int(i * step)})" for i in range(1, NFRAME + 1))
subprocess.run(
    [
        "ffmpeg",
        "-nostdin",
        "-v",
        "error",
        "-i",
        "/tmp/video.mp4",
        "-an",
        "-vf",
        f"select={select}",
        "-fps_mode",
        "passthrough",
        "-q:v",
        "2",
        f"{out}/frames/%02d.jpg",
    ],
    check=True,
)
got = len(os.listdir(f"{out}/frames"))
assert got == NFRAME, f"video {video_id}: extracted {got} of {NFRAME} frames"
print(f"video {video_id}: {n} frames, sampled {NFRAME}")
