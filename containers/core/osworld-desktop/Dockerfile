# OSWorld desktop — the sidecar the agent drives.
#
# This is upstream's own machine: the root filesystem of the VM disk OSWorld
# publishes and scores against, run as a container instead of booted. The guest
# is Ubuntu 22.04 x86_64 and so are the nodes, so nothing is emulated; no
# cluster we run on has /dev/kvm, and software emulation measures 5-24x slower.
#
# Nothing of the desktop is rebuilt here — its apps, their versions and their
# settings are the ones upstream installed. What is added is what a VM has and
# a container does not: a display, and something to start the session.
FROM ubuntu:24.04 AS disk

ARG VM_REVISION=a5d9c3eaae98eebf6e3a0beb84e7e47cf72ae133

# guestfish reads the disk with the kernel's own ext4 driver, in an appliance
# that needs a kernel image to boot.
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
      ca-certificates curl libarchive-tools libguestfs-tools linux-image-virtual \
 && rm -rf /var/lib/apt/lists/*

# Left behind is what only a booted machine uses — device nodes, swap, the
# kernel, the snap and flatpak stores (neither can mount here), the journal —
# and one file whose inode fails its checksum, which upstream's own VM cannot
# read either.
RUN bash -euo pipefail <<'EOF'
curl -fsSL "https://huggingface.co/datasets/xlangai/ubuntu_osworld/resolve/${VM_REVISION}/Ubuntu.qcow2.zip" \
  | bsdtar -xOf - > /vm.qcow2
mkdir /rootfs
LIBGUESTFS_BACKEND=direct guestfish --ro -a /vm.qcow2 -m /dev/sda3 tar-out / - numericowner:true \
  "excludes:./dev/* ./swapfile ./lost+found ./boot/* \
   ./usr/lib/modules/* ./usr/lib/firmware/* ./usr/src/* \
   ./var/lib/snapd/snaps/* ./var/lib/snapd/cache/* ./var/lib/snapd/seed/* \
   ./var/lib/flatpak/* ./var/log/journal/* \
   ./usr/share/code/resources/app/extensions/node_modules/typescript/lib/pl/diagnosticMessages.generated.json" \
  | tar -xp --numeric-owner -C /rootfs
rm /vm.qcow2
EOF

FROM scratch

# Upstream publishes this disk for x86_64 only.
LABEL eval.platforms="linux/amd64"

COPY --from=disk /rootfs/ /

# The VM's display is its virtual graphics card. The stand-in is X's dummy
# driver, configured as upstream documents for a headless machine.
COPY 10-dummy.conf /etc/X11/xorg.conf.d/10-dummy.conf
RUN curl -fsSL -o /tmp/dummy.deb \
      http://archive.ubuntu.com/ubuntu/pool/main/x/xserver-xorg-video-dummy/xserver-xorg-video-dummy_0.3.8-2build1_amd64.deb \
 && echo "2ef60bd404142f7bb3d77a2886ac89d8a0dabc101d87ce7496ebe45f19ae8fc3  /tmp/dummy.deb" | sha256sum -c - \
 && dpkg -i /tmp/dummy.deb \
 && rm /tmp/dummy.deb

# What the VM gets from booting systemd. Its tools look for this directory
# before they will talk to anything. Its login and time daemons are started by
# systemd on request; with none here, D-Bus starts them itself — the session
# cannot start without the first, and `timedatectl`, which one task is graded
# through, needs the second, told there are no NTP units to ask systemd about.
# And without gdm the shell warns once that the screen cannot lock; the last
# line is the marker it leaves after warning.
RUN install -d /run/systemd/system \
 && sed -i 's|^Exec=/bin/false|Exec=/lib/systemd/systemd-logind|' \
      /usr/share/dbus-1/system-services/org.freedesktop.login1.service \
 && sed -i 's|^Exec=/bin/false|Exec=/usr/bin/env SYSTEMD_TIMEDATED_NTP_SERVICES= /lib/systemd/systemd-timedated|' \
      /usr/share/dbus-1/system-services/org.freedesktop.timedate1.service \
 && install -o user -g user -m 0664 /dev/null /home/user/.local/share/gnome-shell/lock-warning-shown

COPY start-desktop.sh agent_face.py /opt/

EXPOSE 5000
CMD ["/opt/start-desktop.sh"]
