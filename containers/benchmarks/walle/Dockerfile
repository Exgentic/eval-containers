# walle — structured-output (response_format) conformance probe. Model-only.
#
# walle (MoonshotAI) is a corpus of JSON Schemas that a model-serving stack
# must handle correctly through its `response_format` / structured-output path.
# It is NOT an agent-capability benchmark — it probes STACK CORRECTNESS: does
# the served endpoint ACCEPT every valid schema and REJECT invalid ones with
# HTTP 400? (Upstream CI phrases it as "checks whether response_format rejects
# [invalid schemas] with HTTP 400".) Used inside MoonshotAI/Kimi-Vendor-Verifier
# to check that a third-party Kimi deployment supports the same JSON-Schema
# surface for constrained decoding as the reference.
#
# 395 cases across 16 feature suites (212 valid + 183 invalid): BasicTypes,
# AnyOf, Defs, References, RangeConstraints, AdditionalProperties, EnforcerCases,
# etc. Each task is one schema plus an `expect` verdict (accept | reject).
#
# Model-only, native-harness (like automationbench): there is no repo agent.
# The runner container holds EVAL_TASK_ID + the gateway endpoint, POSTs the
# schema as `response_format` to the gateway, observes accept (2xx) vs reject
# (400/422), and writes the reward = 1.0 iff the observed verdict matches the
# expected one. No LLM judge, no external services.
#
# Upstream: github.com/MoonshotAI/walle  (MIT)  pinned by commit (tag v0.1.10).

# syntax=docker/dockerfile:1
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint

FROM python:3.12-slim

# Rule 4: one ARG drives both the fetch ref and the data_revision label, so the
# label can never disagree with the data baked in. Upstream tag v0.1.10.
ARG BENCHMARK_VERSION=cc1c6b7dab5496d5184677ecf4c3b95fc1bd1606
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="walle"
LABEL eval.benchmark.description="walle (MoonshotAI) - JSON-Schema response_format conformance probe: does the stack accept valid schemas and reject invalid ones? Stack-correctness, model-only."
LABEL eval.benchmark.tasks="395"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/MoonshotAI/walle"
LABEL eval.benchmark.license="MIT"

# ── apt: ca-certificates for the build-time HTTPS fetch (network-resilient,
# mirrors benchmark-base-external) ────────────────────────────────────────
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing \
         && apt-get install -y --no-install-recommends ca-certificates; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

# ── jsonschema: validates that a valid+accepted case's returned content
# actually conforms (a diagnostic recorded in the export, NOT part of the
# reward — see run_walle.py). Retry loop for transient PyPI flake. ─────────
ENV PIP_DISABLE_PIP_VERSION_CHECK=1
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries=5 --timeout=120 "jsonschema==4.23.0"; \
      then ok=1; break; fi; \
      echo "pip attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "jsonschema install failed after 5 tries" >&2; exit 1; }

# ── Fetch the pinned corpus and assemble the stable index map ─────────────
# The runner selects one case by its sequential integer EVAL_TASK_ID. Assemble
# every valid + invalid schema across the 16 suites in a fixed order (suite
# name × kind[valid,invalid] × line order — no shuffle) and record one row per
# case as /tasks/all.jsonl: {id, suite, kind, expect, schema, upstream_id}.
# `schema` is the raw JSON-Schema TEXT (the original jsonl line) so the shared
# materializer writes it back verbatim (str() of a dict would corrupt it).
# Assert the total so any upstream data drift is caught at build (rule 3).
# Root-only (rule 5): the store must not be agent-readable.
RUN python3 <<'PYEOF'
import json, os, urllib.request

REF = os.environ["BENCHMARK_VERSION"]
BASE = f"https://raw.githubusercontent.com/MoonshotAI/walle/{REF}/testdata/validator_cases"
# The 16 suites present at the pinned ref (stable — the ref is immutable).
SUITES = [
    "TestAdditionalProperties", "TestAnyOf", "TestBasicTypes", "TestDefs",
    "TestDescription", "TestEnforcerCases", "TestID", "TestKeywordsValidation",
    "TestNestedDefsDepth", "TestNumberFormat", "TestRangeConstraints",
    "TestRefInProperties", "TestReferences", "TestRequired",
    "TestSingleTypeInArray", "TestTypeLocation",
]
EXPECT = {"valid": "accept", "invalid": "reject"}

def fetch(suite, kind):
    url = f"{BASE}/{suite}/{kind}.jsonl"
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                return r.read().decode("utf-8")
        except Exception as e:  # noqa: BLE001
            print(f"fetch {url} attempt {attempt+1} failed: {e}")
    raise SystemExit(f"could not fetch {url} after 5 tries")

rows = []
for suite in SUITES:
    for kind in ("valid", "invalid"):
        body = fetch(suite, kind)
        for lineno, line in enumerate(body.splitlines()):
            line = line.strip()
            if not line:
                continue
            obj = json.loads(line)  # every upstream jsonl line is valid JSON
            if kind == "invalid":
                # invalid.jsonl wraps the (possibly non-JSON) malformed schema:
                # {"schema": "<schema text>", "expect": "<why>", "unmarshal": bool}.
                # The case's schema is that INNER text — send THAT, not the
                # wrapper (the wrapper is a well-formed object a stack accepts,
                # which would silently defeat the reject half of the probe).
                schema_text = obj["schema"]
            else:
                # valid.jsonl lines are the schema object itself.
                schema_text = line
            rows.append({
                "id": len(rows),
                "suite": suite,
                "kind": kind,
                "expect": EXPECT[kind],
                "schema": schema_text,
                "upstream_id": f"{suite}:{kind}:{lineno}",
            })

n_valid = sum(r["kind"] == "valid" for r in rows)
n_invalid = sum(r["kind"] == "invalid" for r in rows)
assert len(rows) == 395, f"expected 395 cases at pinned ref, got {len(rows)} ({n_valid} valid + {n_invalid} invalid)"
assert (n_valid, n_invalid) == (212, 183), f"case split drifted: {n_valid} valid + {n_invalid} invalid"

os.makedirs("/tasks", exist_ok=True)
with open("/tasks/all.jsonl", "w") as out:
    for r in rows:
        out.write(json.dumps(r) + "\n")
print(f"Wrote {len(rows)} cases to /tasks/all.jsonl ({n_valid} valid + {n_invalid} invalid)")
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Shared task materializer (writes /tasks/$EVAL_TASK_ID/<field>.txt at runtime).
COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

WORKDIR /app
ENV EVAL_BENCHMARK=walle

# The conformance probe: resolve EVAL_TASK_ID → schema, POST it as
# response_format to the gateway, and write reward = (observed verdict ==
# expected verdict).
COPY run_walle.py /app/run_walle.py
RUN chmod 755 /app/run_walle.py

# The runner writes /logs/verifier/reward.txt itself; keep a fail-closed
# baseline grader so a run that never reaches the writer still scores 0.
RUN mkdir -p /logs/verifier && cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
