# OSWorld-Verified — real desktop for computer-use agents.
#
# The repaired edition of OSWorld's task suite, which is what the public
# leaderboard reports against. Same 369 ids and the same execution-based
# evaluators as the original; 117 of the task definitions carry fixes, so a
# score against the original is not comparable to one against this.
#
# Everything it runs is core/osworld-harness, which osworld-verified-offline is
# built on too; this is the whole of its task list.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}osworld-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld-verified"
LABEL eval.benchmark.description="OSWorld-Verified - the repaired OSWorld suite on a real desktop"
LABEL eval.benchmark.tasks="335"
LABEL eval.benchmark.env="shared-env"
# x86_64 only, as its desktop is. Restated here: the release reads it per image.
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.internet="true"
ENV EVAL_INTERNET=true
ENV EVAL_BENCHMARK=osworld-verified
