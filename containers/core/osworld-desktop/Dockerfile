# OSWorld desktop — the sidecar the agent drives.
#
# Upstream ships this as a 14.9 GB qcow2 booted under QEMU/KVM. This cluster
# has no /dev/kvm, no privileged SCC and no Kata runtime, and software
# emulation measures 5-24x slower (a minimal Ubuntu takes 129s just to boot).
# The guest is plain Ubuntu x86_64 and our nodes are x86_64, so there is no
# architecture gap to bridge: the same desktop runs as an ordinary container
# at native speed, serving upstream's own control API on :5000.
FROM ubuntu:22.04

# Pinned to the same server commit the official VM image provisions
# (xlang-ai/osworld_image group_vars: osworld_server_commit).
ARG OSWORLD_SERVER_COMMIT=a3cc3f0c64e463f020d1a44780307e9b46cbcab1

ARG TARGETARCH
ENV DEBIAN_FRONTEND=noninteractive
ENV DISPLAY=:0

# xdg-utils + shared-mime-info + file: without a MIME association the task's
# own `open` step blocks forever in xdg-open. wmctrl: the grader's
# activate_window step retries until it exists. python: the grader's save step
# shells out to `python`, not `python3`.
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
      xvfb x11-utils xdotool wmctrl scrot \
      xfwm4 xfdesktop4 xfce4-settings dbus-x11 \
      xdg-utils shared-mime-info desktop-file-utils file sudo \
      python3 python3-pip python3-tk python3-dev python3-uno python3-pyatspi \
      at-spi2-core fonts-dejavu git curl ca-certificates gnupg \
      xclip jq unzip xz-utils build-essential socat \
      libreoffice-calc libreoffice-writer libreoffice-impress \
      thunderbird gimp vlc \
 && ln -sf /usr/bin/python3 /usr/bin/python \
 && rm -rf /var/lib/apt/lists/*

RUN pip3 install --no-cache-dir \
      uvicorn fastapi flask websockets numpy lxml pygame \
      python-xlib PyAutoGUI Pillow requests

RUN git clone -q https://github.com/xlang-ai/osworld-server /opt/osworld-server \
 && git -C /opt/osworld-server checkout -q ${OSWORLD_SERVER_COMMIT}

# The tasks address files under /home/user, matching the upstream VM's user.
# OpenShift runs this pod as a random uid whose HOME is `/`, which is not
# writable: LibreOffice and dconf then fail and /setup/open_file 500s. Point
# every writable path at /home/user, which is world-writable, so the image
# works as any uid without needing a privileged service account.
# Task setup shells out to `sudo` (upstream's VM has a user that can), and
# without it a task dies before the agent starts. Nothing is widened by this:
# the agent never reaches a shell here, only screenshots and pyautogui.
# The harness inspects pages over Chrome's DevTools port (Playwright connects
# to 9222), so Chrome has to expose it however it is started — a task that
# opens the browser by clicking would otherwise be ungradeable. Safe Browsing
# is off to match upstream's managed policy, which interferes with test pages.
# --no-sandbox because an eval pod runs as root (the chart's anyuid service
# account) and Chrome refuses to start as root without it — upstream's VM runs
# it as an ordinary user, so the flag is this container's problem, not theirs.
# Nothing is weakened that the pod does not already have: the agent reaches
# Chrome only through pyautogui on a screen.
RUN [ "${TARGETARCH:-amd64}" = "amd64" ] || exit 0; \
    mkdir -p /etc/opt/chrome/policies/managed \
 && printf '{"SafeBrowsingProtectionLevel":0,"SafeBrowsingEnabled":false}\n' \
      > /etc/opt/chrome/policies/managed/osworld-safe-browsing.json \
 && printf '%s\n' \
      '#!/bin/sh' \
      '# Only default the port when the caller did not ask for one: Chrome' \
      '# honours the FIRST --remote-debugging-port, so forcing ours here' \
      '# silently ignored a task that asked for its own and bridged to it.' \
      'case " $* " in' \
      '  *" --remote-debugging-port="*) p= ;;' \
      '  *) p=--remote-debugging-port=9222 ;;' \
      'esac' \
      'exec /opt/google/chrome/google-chrome $p --remote-allow-origins=* --no-sandbox --no-first-run --no-default-browser-check "$@"' \
      > /usr/local/bin/google-chrome \
 && chmod +x /usr/local/bin/google-chrome \
 && ln -sf /usr/local/bin/google-chrome /usr/local/bin/google-chrome-stable

RUN echo "ALL ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/osworld \
 && chmod 440 /etc/sudoers.d/osworld

ENV HOME=/home/user
ENV XDG_CONFIG_HOME=/home/user/.config
ENV XDG_CACHE_HOME=/home/user/.cache
ENV XDG_DATA_HOME=/home/user/.local/share
ENV XDG_RUNTIME_DIR=/tmp/runtime
RUN mkdir -p /home/user/.config /home/user/.cache /home/user/.local/share /tmp/runtime \
 && chmod -R 777 /home/user /tmp/runtime

# The desktop this replaces is upstream's Ubuntu 22.04 *desktop* image, and
# their Ansible is only a delta over it — so the apps the tasks drive
# (thunderbird, gimp, vlc above; chrome and vs code below) are assumed by every
# task, not installed by the playbook. A task whose app is absent scores 0 for a
# reason that has nothing to do with the agent, so they are installed here.
# VS Code refuses to start as root as well, and wants its data directory named
# explicitly when it does, so it gets a wrapper like Chrome's — written after
# its own deb, which removes one written before it. The path is the one VS Code
# would pick anyway, so a non-root run is unchanged.
# Chrome and VS Code ship amd64 debs only, and the fleet builds this image for
# both arches. The cluster is amd64, so they are installed there and skipped on
# arm64 (a Mac dev build), rather than failing the whole release.
# Every app check runs as a foreign uid, because that is what the pod gets:
# root's `command -v` is happy with a tree the agent's uid cannot traverse, and
# an app the agent cannot open scores the task 0 exactly like a failed attempt.
RUN cat > /usr/local/bin/check-apps <<'CHECK' && chmod +x /usr/local/bin/check-apps
#!/bin/sh
for b in "$@"; do
  setpriv --reuid=12345 --regid=0 --clear-groups sh -c "command -v '$b' >/dev/null" \
    || { echo "osworld-desktop: $b is not usable by the pod's uid"; exit 1; }
done
CHECK

RUN [ "${TARGETARCH:-amd64}" = "amd64" ] || exit 0; \
    install -d -m 0755 /etc/apt/keyrings \
 && curl -fsSL https://dl.google.com/linux/linux_signing_key.pub \
      | gpg --dearmor -o /etc/apt/keyrings/google-chrome.gpg \
 && echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/google-chrome.gpg] http://dl.google.com/linux/chrome/deb/ stable main" \
      > /etc/apt/sources.list.d/google-chrome.list \
 && curl -fsSL https://packages.microsoft.com/keys/microsoft.asc \
      | gpg --dearmor -o /etc/apt/keyrings/microsoft.gpg \
 && echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/microsoft.gpg] https://packages.microsoft.com/repos/code stable main" \
      > /etc/apt/sources.list.d/vscode.list \
 && apt-get update -qq \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
      google-chrome-stable code \
 && rm -rf /var/lib/apt/lists/* \
 && printf '#!/bin/sh\nexec /usr/bin/code --no-sandbox \\\n  --user-data-dir="${HOME:-/home/user}/.config/Code" "$@"\n' \
      > /usr/local/bin/code \
 && chmod +x /usr/local/bin/code \
 && [ "$(command -v code)" = /usr/local/bin/code ] \
 && sed -i 's#^Exec=/usr/bin/google-chrome-stable#Exec=/usr/local/bin/google-chrome#' \
      /usr/share/applications/google-chrome.desktop \
 && grep -q '^Exec=/usr/local/bin/google-chrome' /usr/share/applications/google-chrome.desktop \
 && /usr/local/bin/check-apps google-chrome code soffice thunderbird gimp vlc

# The other half of what upstream's snapshot carries: a home directory the apps
# have already run in. See warm-apps.sh.
COPY warm-apps.sh /usr/local/bin/warm-apps.sh
RUN [ "${TARGETARCH:-amd64}" = "amd64" ] || exit 0; \
    /usr/local/bin/warm-apps.sh

RUN for m in application/vnd.openxmlformats-officedocument.spreadsheetml.sheet \
             application/vnd.ms-excel text/csv \
             application/vnd.oasis.opendocument.spreadsheet \
             application/vnd.openxmlformats-officedocument.wordprocessingml.document \
             application/vnd.openxmlformats-officedocument.presentationml.presentation; do \
      xdg-mime default libreoffice-calc.desktop "$m" || true; \
    done; \
    update-mime-database /usr/share/mime >/dev/null 2>&1 || true; \
    update-desktop-database /usr/share/applications >/dev/null 2>&1 || true

RUN cat > /start-desktop.sh <<'START' && chmod +x /start-desktop.sh
#!/bin/bash
set -x
Xvfb :0 -screen 0 "${SCREEN_WIDTH:-1920}x${SCREEN_HEIGHT:-1080}x24" >/tmp/xvfb.log 2>&1 &
for i in $(seq 1 30); do xdpyinfo -display :0 >/dev/null 2>&1 && break; sleep 1; done
dbus-launch xfwm4 >/tmp/wm.log 2>&1 &
xfdesktop >/tmp/xfdesktop.log 2>&1 &
sleep 2

# One LibreOffice instance for later `open` steps to attach to: without an
# endpoint to attach through, a second soffice blocks on the profile lock and
# the document never opens, which is what /setup/open_file waits for.
# A named pipe, not the usual `socket,port=2002`: a port is on the localhost
# the agent's container shares, and UNO can open, edit and save a document —
# the work the agent is meant to do on screen. A pipe is a file in this
# container, which the agent has no mount for.
setsid /usr/bin/libreoffice --norestore --nologo \
  --accept="pipe,name=osworld;urp;StarOffice.ServiceManager" \
  >/tmp/lo.log 2>&1 < /dev/null &
sleep 8

# The real server on a unix socket only root can open: setup and grading use
# it, and the agent cannot, because it shares the pod's network namespace and a
# socket is not on the network. The port carries the restricted face instead.
mkdir -p /run/desktop
# The runner shares this directory, and unlink needs the directory, not
# the socket: without the sticky bit the agent can delete root's socket
# and bind its own, then answer the grader.
chmod 1777 /run/desktop
cd /opt/osworld-server
python3 -m uvicorn src.server:asgi_app --uds /run/desktop/desktop.sock >/tmp/server.log 2>&1 &
for i in $(seq 1 60); do [ -S /run/desktop/desktop.sock ] && break; sleep 1; done
chmod 600 /run/desktop/desktop.sock
exec python3 /opt/agent_face.py
START

COPY agent_face.py /opt/agent_face.py

EXPOSE 5000
CMD ["/start-desktop.sh"]
