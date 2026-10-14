# OSWorld-Verified, offline — the tasks that need nothing from the internet.
#
# core/osworld-harness, as osworld-verified is, with the task list cut to the 251
# tasks that can be set up, done and graded with the network cut (offline.txt;
# osworld-verified's AUDIT.md says how each of the other 84 was ruled out).
# Nothing else differs: the same harness, the same desktop, the same graders.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}osworld-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld-verified-offline"
LABEL eval.benchmark.description="OSWorld-Verified - the tasks that need nothing from the internet"
LABEL eval.benchmark.tasks="251"
LABEL eval.benchmark.env="shared-env"
# x86_64 only, as its desktop is. Restated here: the release reads it per image.
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ENV EVAL_BENCHMARK=osworld-verified-offline

# The tasks keep the order they have in osworld-verified, so a task's number
# here is its rank among the 251, not its number there; id.txt is the same.
COPY offline.txt /tmp/offline.txt
RUN python3 <<'PYEOF' && rm /tmp/offline.txt
import json

keep = set(open('/tmp/offline.txt').read().split())
rows = [line for line in open('/tasks/all.jsonl') if json.loads(line)['id'] in keep]
if not len(rows) == len(keep) == 251:
    raise SystemExit('osworld-verified-offline: expected 251 tasks, got %d of %d' % (len(rows), len(keep)))
open('/tasks/all.jsonl', 'w').writelines(rows)
print('Kept %d tasks' % len(rows))
PYEOF
