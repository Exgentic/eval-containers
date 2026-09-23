#!/bin/bash
# Open every app once, so the image ships a home directory that has been used.
#
# Upstream's snapshot is a desktop someone has already worked on: each app has
# run at least once, written its profile and had its first-run wizard dismissed.
# A container starts from a home no app has ever seen, so the agent's first
# click lands on a welcome dialog the task never accounted for — and the task
# is scored as if the agent failed it.
#
# Run at build time, as root; the profiles land in /home/user, which every uid
# can write (OpenShift assigns a random one).
set -eu
export HOME=/home/user DISPLAY=:99
# Not the image's /tmp/runtime: the XDG spec says a runtime dir is 0700 and
# owned by its user, so the first GTK app here would lock it to root and the
# pod's own uid would lose it. Warm in a throwaway one instead.
export XDG_RUNTIME_DIR=/tmp/warm-runtime
mkdir -p "$XDG_RUNTIME_DIR"
: "${WARM_SECONDS:=25}"

Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp >/dev/null 2>&1 &
for _ in $(seq 1 100); do xdpyinfo >/dev/null 2>&1 && break; sleep 0.2; done
xdpyinfo >/dev/null 2>&1 || { echo "warm-apps: X never came up"; exit 1; }
xfwm4 --daemon >/dev/null 2>&1 || true

# Closed by process group, not by name: half of these are AppImages or wrappers
# whose real process is named something else entirely, and a pkill pattern wide
# enough to catch them is wide enough to kill the build step.
warm() {  # warm <command>...
  echo "warm: $1"
  setsid "$@" >/tmp/warm.log 2>&1 &
  local pid=$!          # setsid makes the child a session leader: pid == pgid
  sleep "$WARM_SECONDS"
  # An app that is already gone MAY never have drawn a window — Zotero spent a
  # release like that, its libxul unable to find libdbus-glib, dying in a
  # second while the build and every `command -v` stayed happy. Read the lines,
  # do not trust the fact: a launcher that spawns and returns (`code`) is gone
  # here too and perfectly healthy.
  kill -0 "$pid" 2>/dev/null || sed 's/^/    died: /' /tmp/warm.log | head -5
  kill -TERM -"$pid" 2>/dev/null || true
  sleep 2
  kill -KILL -"$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}

# LibreOffice and GIMP have a batch mode that writes the profile without a
# window, which is faster and cannot hang on a dialog.
soffice --headless --terminate_after_init >/dev/null 2>&1 || true
gimp -i -b '(gimp-quit 0)' >/dev/null 2>&1 || true
thunderbird -CreateProfile default >/dev/null 2>&1 || true
code --list-extensions >/dev/null 2>&1 || true

for app in "soffice --writer" \
           "thunderbird" \
           "google-chrome --no-first-run --no-default-browser-check about:blank" \
           "code --disable-gpu" \
           "zotero" "wps" "et" "wpp" "shotcut" "musescore" "obsidian" \
           "freecad" "kicad" "blender" "nautilus" "evince" "eog"; do
  # The flags above are meant to split into separate arguments.
  # shellcheck disable=SC2086
  warm $app
done

pkill -x Xvfb >/dev/null 2>&1 || true

# A profile that was open when the build ended still says so. Chrome refuses to
# reuse one locked by "another computer" (the build host), and the mozilla-family
# apps hold a .parentlock — every one of those is a dialog the agent would meet
# instead of the app.
find "$HOME" -maxdepth 5 \( -name 'Singleton*' -o -name '.parentlock' -o -name 'lock' \) \
  -delete 2>/dev/null || true

# What each app left behind. Read this in the build log: an app that writes
# nothing never started, and its first window at run time will be a wizard.
echo "warm-apps: state written"
( cd "$HOME" && find . -maxdepth 2 -mindepth 1 \
    \( -name '.cache' -o -name '.dbus' \) -prune -o -print ) | sort | sed 's/^/  /'

# Written by root; the runtime uid is random and must still own its own desktop.
chmod -R 777 /home/user
rm -rf "$XDG_RUNTIME_DIR"
