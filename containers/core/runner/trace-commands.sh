# shellcheck shell=bash
# Sourced via BASH_ENV by every non-interactive bash the agent starts, so the
# commands it runs land in /output/agent/console.log. Set on the agent launch in
# /usr/local/bin/run-agent, which also creates the file.
#
# Each guard here exists because the alternative sends the trace to STDERR, where
# the agent's tooling hands it to the model as tool output and the recorder
# becomes a prompt change:
#   - a `{var}` descriptor is close-on-exec, so a nested `bash -c` would inherit
#     an exported variable but not the open file — hence one open per shell;
#   - an empty BASH_XTRACEFD does not disable tracing, so it is set only once the
#     descriptor exists.
# The version check is separate: bash < 4.1 reads `{_EVAL_TRACE_FD}` as a command
# and exits 127, killing the agent. PS4 avoids `date` because it is expanded
# before every command.
if [ -w /output/agent/console.log ] && [ "${BASH_VERSINFO[0]:-0}" -ge 5 ]; then
  exec {_EVAL_TRACE_FD}>>/output/agent/console.log || true
  if [ -n "${_EVAL_TRACE_FD:-}" ]; then
    BASH_XTRACEFD=$_EVAL_TRACE_FD
    PS4='+ ${EPOCHREALTIME} '
    set -x                    # last, so none of the above is traced
  fi
fi
