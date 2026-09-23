# OSWorld desktop — the sidecar the agent drives.
#
# Upstream ships this as a 14.9 GB qcow2 booted under QEMU/KVM. This cluster
# has no /dev/kvm, no privileged SCC and no Kata runtime, and software
# emulation measures 5-24x slower (a minimal Ubuntu takes 129s just to boot).
# The guest is plain Ubuntu x86_64 and our nodes are x86_64, so there is no
# architecture gap to bridge: the same desktop runs as an ordinary container
# at native speed, serving upstream's own control API on :5000.
FROM docker.io/library/ubuntu:22.04

# Pinned to the same server commit the official VM image provisions
# (xlang-ai/osworld_image group_vars: osworld_server_commit).
ARG OSWORLD_SERVER_COMMIT=a3cc3f0c64e463f020d1a44780307e9b46cbcab1

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
      xclip jq unzip xz-utils build-essential \
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
RUN [ "${TARGETARCH:-amd64}" = "amd64" ] || exit 0; \
    mkdir -p /etc/opt/chrome/policies/managed \
 && printf '{"SafeBrowsingProtectionLevel":0,"SafeBrowsingEnabled":false}\n' \
      > /etc/opt/chrome/policies/managed/osworld-safe-browsing.json \
 && printf '#!/bin/sh\nexec /opt/google/chrome/google-chrome \\\n  --remote-debugging-port=9222 --remote-allow-origins=* \\\n  --no-first-run --no-default-browser-check "$@"\n' \
      > /usr/local/bin/google-chrome \
 && chmod +x /usr/local/bin/google-chrome \
 && ln -sf /usr/local/bin/google-chrome /usr/local/bin/google-chrome-stable \
 && sed -i 's#^Exec=/usr/bin/google-chrome-stable#Exec=/usr/local/bin/google-chrome#' \
      /usr/share/applications/google-chrome.desktop || true

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
# Chrome and VS Code ship amd64 debs only, and the fleet builds this image for
# both arches. The cluster is amd64, so they are installed there and skipped on
# arm64 (a Mac dev build), rather than failing the whole release.
ARG TARGETARCH
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
 && rm -rf /var/lib/apt/lists/*

# The rest of upstream's app set, at the versions their Ansible pins. WPS,
# Shotcut, MuseScore and Zotero are what the apps-subset tasks start in
# (snapshot = wps/shotcut/musescore/zotero), and a missing one scores 0 for a
# reason that has nothing to do with the agent.
#
# AppImages are extracted rather than run: mounting one needs FUSE, which a
# container does not have. Zotero comes from the tarball because upstream
# installs it as a snap and snapd does not run in a container.
ARG WPS_DEB=https://wdl1.pcfg.cache.wpscdn.com/wpsdl/wpsoffice/download/linux/11723/wps-office_11.1.0.11723.XA_amd64.deb
ARG SHOTCUT_APPIMAGE=https://downloads.sourceforge.net/project/shotcut/v26.2.26/shotcut-linux-x86_64-26.2.26.AppImage
ARG MUSESCORE_APPIMAGE=https://github.com/musescore/MuseScore/releases/download/v4.6.0/MuseScore-Studio-4.6.0.252730944-x86_64.AppImage
ARG ZOTERO_TARBALL=https://download.zotero.org/client/release/7.0.11/Zotero-7.0.11_linux-x86_64.tar.bz2
RUN [ "${TARGETARCH:-amd64}" = "amd64" ] || exit 0; \
    set -eux; mkdir -p /opt/appimages; \
    curl -fsSL -o /tmp/wps.deb "$WPS_DEB" \
      && (dpkg -i /tmp/wps.deb \
          || apt-get install -y -qq --no-install-recommends -f) \
      && rm /tmp/wps.deb; \
    for a in "$SHOTCUT_APPIMAGE" "$MUSESCORE_APPIMAGE"; do \
      n=$(basename "$a" | cut -d- -f1 | tr 'A-Z' 'a-z'); \
      curl -fsSL -o /tmp/app.AppImage "$a" && chmod +x /tmp/app.AppImage; \
      (cd /opt/appimages && /tmp/app.AppImage --appimage-extract >/dev/null \
        && mv squashfs-root "$n"); \
      ln -sf "/opt/appimages/$n/AppRun" "/usr/local/bin/$n"; rm /tmp/app.AppImage; \
    done; \
    curl -fsSL -o /tmp/zotero.tar.bz2 "$ZOTERO_TARBALL" \
      && mkdir -p /opt/zotero && tar -xjf /tmp/zotero.tar.bz2 -C /opt/zotero --strip-components=1 \
      && ln -sf /opt/zotero/zotero /usr/local/bin/zotero && rm /tmp/zotero.tar.bz2; \
    rm -rf /var/lib/apt/lists/*

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

# LibreOffice is started once with a UNO socket so later `open` steps attach to
# this process; nothing else depends on the socket, it just keeps one instance.
setsid /usr/bin/libreoffice --norestore --nologo \
  --accept="socket,host=localhost,port=2002;urp;StarOffice.ServiceManager" \
  >/tmp/lo.log 2>&1 < /dev/null &
sleep 8

# The real server on a unix socket only root can open: setup and grading use
# it, and the agent cannot, because it shares the pod's network namespace and a
# socket is not on the network. The port carries the restricted face instead.
mkdir -p /run/desktop
cd /opt/osworld-server
python3 -m uvicorn src.server:asgi_app --uds /run/desktop/desktop.sock >/tmp/server.log 2>&1 &
for i in $(seq 1 60); do [ -S /run/desktop/desktop.sock ] && break; sleep 1; done
chmod 600 /run/desktop/desktop.sock
exec python3 /opt/agent_face.py
START

COPY agent_face.py /opt/agent_face.py

EXPOSE 5000
CMD ["/start-desktop.sh"]
