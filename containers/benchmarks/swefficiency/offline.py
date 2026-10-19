"""Make one SWE-fficiency task gradable offline (README.md, "Offline").

Pip's isolated builds fetch the repository's build requirements: download them to
/opt/wheelhouse. A workload that downloads data with wget on every run: fetch the
data once and let /usr/local/bin/wget serve it.
"""

import hashlib
import json
import re
import subprocess
import tomllib
import urllib.request
from pathlib import Path

pyproject = Path("/testbed/pyproject.toml")
requires = (
    tomllib.loads(pyproject.read_text()).get("build-system", {}).get("requires", [])
    if pyproject.exists()
    else []
)
Path("/opt/wheelhouse").mkdir()
if requires:
    subprocess.run(
        [
            "/opt/miniconda3/envs/testbed/bin/pip",
            "download",
            "-q",
            "--dest",
            "/opt/wheelhouse",
            *requires,
        ],
        check=True,
    )

workload = json.loads(Path("/tasks/0/config.json").read_text())["workload"]
urls = re.findall(r"https?://[^\s'\"]+", workload) if "'wget'" in workload else []
for url in urls:
    data = Path("/opt/workload-data") / hashlib.sha256(url.encode()).hexdigest()
    data.parent.mkdir(exist_ok=True)
    urllib.request.urlretrieve(url, data)
if urls:
    Path("/usr/local/bin/wget").write_text(
        "#!/bin/bash\n# Serves the data a workload downloads, fetched at build (offline.py).\n"
        'for a in "$@"; do f=/opt/workload-data/$(printf %s "$a" | sha256sum | cut -d" " -f1); [ -f "$f" ] && src=$f; done\n'
        'while [ $# -gt 0 ]; do [ "$1" = -O ] && out=$2; shift; done\n'
        'cp "$src" "$out"\n'
    )
    Path("/usr/local/bin/wget").chmod(0o755)
