#!/bin/bash
# Start upstream's desktop the way its own boot does. Two things a container
# cannot have are stood in for: systemd, whose part here is the system bus, and
# a display manager, whose part is starting the X server and the session.
# Everything else — the session, the apps, the control server — is upstream's,
# started with the environment upstream starts it with.
set -eu
trap 'exit 0' TERM

install -d /run/dbus
install -d -m 0700 -o user -g user /run/user/1000
install -d -m 1777 /tmp/.X11-unix /tmp/.ICE-unix
# The runner shares this directory to reach the control server as root. Only
# the desktop's own user may enter it, so the agent's uid can neither open the
# socket nor replace it.
install -d -m 0700 -o user -g user /run/desktop
rm -f /run/desktop/desktop.sock
dbus-daemon --system --fork

. /etc/default/locale
as_user=(runuser -u user -- env -i HOME=/home/user USER=user LOGNAME=user SHELL=/bin/bash
  LANG="$LANG" DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/1000
  DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus)
# What gdm adds for the session it logs in.
session=("${as_user[@]}" USERNAME=user XDG_SESSION_TYPE=x11 XDG_SESSION_CLASS=user
  XDG_SESSION_DESKTOP=ubuntu XDG_CURRENT_DESKTOP=ubuntu:GNOME DESKTOP_SESSION=ubuntu GDMSESSION=ubuntu
  PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin)
# What systemd gives osworld.service.
service=("${as_user[@]}" PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin)
cd /home/user

"${session[@]}" /usr/lib/xorg/Xorg :0 -noreset -nolisten tcp -logfile /tmp/Xorg.0.log &
until [ -S /tmp/.X11-unix/X0 ]; do sleep 0.2; done
"${session[@]}" dbus-daemon --session --address=unix:path=/run/user/1000/bus --fork --nopidfile
"${session[@]}" pulseaudio --start
"${session[@]}" /etc/gdm3/Xsession \
  "env GNOME_SHELL_SESSION_MODE=ubuntu /usr/bin/gnome-session --session=ubuntu --builtin" &
until "${session[@]}" xprop -root _NET_SUPPORTING_WM_CHECK 2>/dev/null | grep -q 'window id'; do sleep 0.5; done

# Upstream's control server, unmodified, on a socket instead of a port: the
# agent shares this pod's network, and /execute is a shell. The port carries
# agent_face, which is all the agent may reach.
"${service[@]}" /usr/bin/python -c '
import sys
sys.path.insert(0, "/home/user/server")
from main import app
app.run(host="unix:///run/desktop/desktop.sock")' &
"${service[@]}" /usr/bin/python3 /opt/agent_face.py &

# Stay as pid 1 so exited apps are reaped, and stop when any piece dies.
wait -n
