#!/bin/bash
# Replace vulnerable Log4j jars (< 2.17.1: CVE-2021-44228, -45046, -45105, -44832)
# in the upstream base image with the pinned 2.17.1 release.
#
# Some per-PR bases (rocket-chip, XiangShan) pre-fetch the sbt/Mill toolchain their
# tb_script runs, leaving log4j 2.8.1-2.11.2 in ~/.sbt/boot, ~/.ivy2/cache and the
# coursier caches. Grading is offline, so those tools must keep resolving the
# cached coordinates: each jar keeps its path and only its content is swapped, and
# coursier's checksum sidecars are rewritten to match. Jars already at or above
# 2.17.1 are left alone; bases without Log4j are untouched.
set -euo pipefail

FIXED=2.17.1
MAVEN=https://repo1.maven.org/maven2/org/apache/logging/log4j
declare -A SHA256=(
  [log4j-api-2.17.1.jar]=b0d8a4c8ab4fb8b1888d0095822703b0e6d4793c419550203da9e69196161de4
  [log4j-core-2.17.1.jar]=c967f223487980b9364e94a7c7f9a8a01fd3ee7c19bdbf0b0f9f8cb8511f3d41
  [log4j-core-2.17.1-tests.jar]=3aaa761991c98aa1dff6f07b3f8f6b910a904e4f66253e39025fcd31c51ae423
  [log4j-slf4j-impl-2.17.1.jar]=e9a03720e5d5076009c2530635da9d08485e28a0b0ec20708dadc51afb78e41e
)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

find / -xdev -type f -regextype posix-extended \
  -regex '.*/log4j-(api|core|slf4j-impl)-[0-9][0-9.]*(-tests)?\.jar' -print0 |
  while IFS= read -r -d '' jar; do
    name=${jar##*/}
    [[ $name =~ ^(log4j-[a-z0-9-]+)-([0-9.]+)(-tests)?\.jar$ ]]
    art=${BASH_REMATCH[1]} ver=${BASH_REMATCH[2]} tests=${BASH_REMATCH[3]}
    [[ $(printf '%s\n' "$FIXED" "$ver" | sort -V | head -n1) == "$FIXED" ]] && continue
    new="$tmp/$art-$FIXED$tests.jar"
    if [[ ! -f $new ]]; then
      curl -fsSL --retry 3 -o "$new" "$MAVEN/$art/$FIXED/${new##*/}"
      echo "${SHA256[${new##*/}]}  $new" | sha256sum -c --quiet -
    fi
    cp "$new" "$jar"
    dir=${jar%/*}
    for sum in sha1 md5; do
      side="$dir/.${name}__$sum"
      [[ -f $side ]] && "${sum}sum" <"$new" | cut -d' ' -f1 | tr -d '\n' >"$side"
    done
    rm -f "$dir/.${name}__sha1.computed"
    echo "log4j: $jar $ver -> $FIXED"
  done
