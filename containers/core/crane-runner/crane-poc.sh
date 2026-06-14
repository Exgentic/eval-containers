#!/bin/sh
# Daemonless proof for the crane runner. Run it inside a plain Linux container --
# that container models the eval pod; nothing below talks to a Docker daemon:
#
#   docker run --rm -i debian:stable-slim sh < crane-poc.sh
#
set -e

echo "=== environment: a plain Linux box (the 'eval pod') ==="
if command -v docker >/dev/null 2>&1; then echo "docker daemon in here: PRESENT (unexpected!)"; else echo "docker daemon in here: NONE  <-- so nothing below is DinD"; fi
echo

echo "[setup] fetch crane + bwrap (userspace tools, no daemon)..."
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq --no-install-recommends ca-certificates curl bubblewrap >/dev/null 2>&1
case "$(uname -m)" in x86_64) ca=x86_64; pa=amd64 ;; aarch64|arm64) ca=arm64; pa=arm64 ;; *) ca="$(uname -m)"; pa="$ca" ;; esac
curl -sSL "https://github.com/google/go-containerregistry/releases/latest/download/go-containerregistry_Linux_${ca}.tar.gz" \
  | tar -xz -C /usr/local/bin crane
echo "  crane=$(crane version 2>/dev/null || echo ok)  bwrap=$(command -v bwrap)  arch=${pa}"
echo

echo "[1] DAEMONLESS PULL  (this is the part DinD would otherwise do)"
echo "    crane export docker://alpine  ->  a root filesystem, just download+untar"
mkdir -p /bench
crane export --platform "linux/${pa}" alpine:latest - | tar -x -C /bench
echo "    got rootfs: $(du -sh /bench 2>/dev/null | cut -f1),  top: $(ls /bench | tr '\n' ' ')"
echo

echo "[2] RUN a binary FROM the pulled rootfs  (chroot, as root, no daemon)"
printf '    /etc/alpine-release inside the pulled fs: '
chroot /bench /bin/sh -c 'cat /etc/alpine-release'
echo

echo "[3] COMPOSE (the universal-runner trick): drop an 'agent' into the benchmark"
echo "    rootfs and run it there -- it reads and edits the testbed filesystem"
printf '%s\n' \
  '#!/bin/sh' \
  'echo "      agent is running INSIDE the benchmark fs (alpine $(cat /etc/alpine-release))"' \
  ': > /patch.diff && echo "      agent edited the testbed: $(ls -la /patch.diff)"' \
  > /bench/agent.sh
chroot /bench /bin/sh /agent.sh
echo
echo "=== PROVEN: pull rootfs (crane) + run in it (chroot/bwrap) = no daemon, no DinD ==="
