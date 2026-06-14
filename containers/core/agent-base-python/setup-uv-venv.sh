#!/bin/bash
# Shared helper for agents whose upstream pins Python >=3.x. Subclasses
# call it from their install.sh:
#
#   /opt/agent/setup-uv-venv.sh /opt/<name>-venv pkg1==X pkg2==Y
#
# Optional `--python <version>` precedes the venv path to override the
# default of 3.12:
#
#   /opt/agent/setup-uv-venv.sh --python 3.13 /opt/<name>-venv pkg==X
#
# UV_PYTHON_INSTALL_DIR lands the managed Python in /opt (not /root,
# which the combo image hardens to 0700) so the agent uid 1002 can
# traverse the venv's symlink chain at runtime. `chmod -R a+rX` is
# defensive — uv defaults are usually agent-readable, but the combo
# image sometimes runs umask 0077 during build.
set -euo pipefail

python_version=3.12
if [ "${1:-}" = "--python" ]; then
  python_version=$2
  shift 2
fi
venv_path=$1
shift

export UV_PYTHON_INSTALL_DIR=/opt/uv-python
command -v uv >/dev/null 2>&1 || pip install --no-cache-dir --quiet uv
uv python install --quiet "$python_version"
uv venv --quiet --python "$python_version" "$venv_path"
uv pip install --quiet --python "$venv_path/bin/python" "$@"
chmod -R a+rX /opt/uv-python "$venv_path"
