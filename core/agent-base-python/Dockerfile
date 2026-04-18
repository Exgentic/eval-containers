# syntax=docker/dockerfile:1
# Shared base for python-runtime agents (Rule 11a).
# Provides:
#   - apt-retry wrapper (network flake defence)
#   - uv as the primary Python package manager
#   - a `/usr/local/bin/pip` shim that forwards to `uv pip` so subclass
#     `RUN pip install PACKAGE==X.Y.Z` calls transparently use uv's
#     parallel fetch + connection reuse — no subclass edits required
#   - /opt/agent skeleton (install.sh, entrypoint.sh) that subclasses
#     override
FROM --platform=linux/amd64 python:3.12-slim
LABEL dock.type="core-base"
LABEL dock.base.runtime="python"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates xz-utils git; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

# uv — faster + more resilient to flaky networks than stock pip. Install
# into /usr/local/bin so it's on PATH for every RUN layer downstream.
# uv via PyPI (more reliable than astral.sh's redirect chain to github).
# Pin version for reproducibility. Retry loop for transient PyPI flake.
# NB: this uses the REAL pip (before the shim below is installed) to
# bootstrap uv, then the shim kicks in for all subclass `pip install`.
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries 5 --timeout 60 uv==0.5.14; then ok=1; break; fi; \
      echo "pip install uv attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "pip install uv failed after 5 tries" >&2; exit 1; }

# pip shim: every subclass `pip install ...` call re-routes to `uv pip`
# with a 5-shot retry loop. `UV_SYSTEM_PYTHON=1` targets the image's
# system Python (no venv), matching pip's default behaviour inside a
# container. Pip-only flags uv doesn't accept (`--retries`, `--timeout`)
# are stripped. `UV_HTTP_TIMEOUT=120` lifts uv's 30s default.
#
# Retry rationale: podman-on-macOS has flaky network during concurrent
# image builds — `uv pip install` metadata fetches to files.pythonhosted.org
# stall. Without this loop, one blip kills a subclass's image build.
# One shim, every subclass inherits the robustness (rule 11).
ENV UV_SYSTEM_PYTHON=1
ENV UV_HTTP_TIMEOUT=120
RUN cat > /usr/local/bin/pip <<'SHIM' && chmod +x /usr/local/bin/pip
#!/bin/bash
# Dock pip shim → uv pip with retry (core/agent-base-python).
args=()
skip_next=0
for a in "$@"; do
  if [ "$skip_next" = 1 ]; then skip_next=0; continue; fi
  case "$a" in
    --retries|--timeout) skip_next=1 ;;
    --retries=*|--timeout=*) ;;
    *) args+=("$a") ;;
  esac
done
for i in 1 2 3 4 5; do
  if uv pip "${args[@]}"; then exit 0; fi
  echo "[dock-pip-shim] uv pip attempt $i/5 failed; retry in $((i*5))s" >&2
  sleep $((i*5))
done
echo "[dock-pip-shim] uv pip failed after 5 tries" >&2
exit 1
SHIM

RUN mkdir -p /opt/agent && \
    printf '#!/bin/bash\nset -euo pipefail\n# Default skeleton: subclass agent Dockerfiles overwrite this.\n' > /opt/agent/install.sh && chmod +x /opt/agent/install.sh && \
    printf '#!/bin/bash\necho "ERROR: agent-base entrypoint not overridden by subclass" >&2\nexit 1\n' > /opt/agent/entrypoint.sh && chmod +x /opt/agent/entrypoint.sh
