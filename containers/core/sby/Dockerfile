# SymbiYosys (sby), the front end for Yosys's formal flows, ready to COPY.
#
# Not packaged by Debian, and pure Python: this is upstream's own `make install`
# (its Makefile at the tag) into /bundle, which images copy to / beside the
# Debian yosys, z3 and python3-click it runs on. Its version follows Yosys's,
# so it is pinned to the Debian trixie yosys (0.52) the images install.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-github:latest AS fetch
ARG SBY_VERSION=0.52
RUN curl -fsSL "https://github.com/YosysHQ/sby/archive/refs/tags/v${SBY_VERSION}.tar.gz" | tar -xz -C /tmp \
 && cd "/tmp/sby-${SBY_VERSION}" \
 && L=/bundle/usr/local/share/yosys/python3 && mkdir -p "$L" /bundle/usr/local/bin \
 && cp sbysrc/sby_*.py "$L/" \
 && sed -e 's|##yosys-program-prefix##|""|' sbysrc/sby_core.py > "$L/sby_core.py" \
 && sed -e 's|##yosys-sys-path##|sys.path += [os.path.dirname(__file__) + p for p in ["/share/python3", "/../share/yosys/python3"]]|' \
        -e "s|##yosys-release-version##|release_version = 'SBY v${SBY_VERSION}'|" \
        -e '1s|.*|#!/usr/bin/python3|' \
        sbysrc/sby.py > /bundle/usr/local/bin/sby \
 && chmod +x /bundle/usr/local/bin/sby \
 && install -D -m 644 COPYING /bundle/usr/local/share/doc/sby/COPYING

FROM scratch
LABEL eval.type="core-base"
LABEL eval.base.kind="sby"
COPY --from=fetch /bundle /bundle
