#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-dev}"
RUNTIME="${2:-linux-x64}"
ARCH="${3:-amd64}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STAGE_DIR="$REPO_ROOT/build/package/EtcdPilot-$VERSION-$RUNTIME"
OUT_DIR="$REPO_ROOT/artifacts"
CONFIG="$REPO_ROOT/build/nfpm-$RUNTIME.yaml"

if [ ! -d "$STAGE_DIR" ]; then
  echo "Portable package stage directory not found: $STAGE_DIR" >&2
  exit 1
fi

mkdir -p "$OUT_DIR" "$REPO_ROOT/build"

cat > "$CONFIG" <<EOF
name: etcdpilot
arch: $ARCH
platform: linux
version: ${VERSION#v}
section: default
priority: optional
maintainer: EtcdPilot
description: Lightweight etcd management console
homepage: https://github.com/wanghao9103/EtcdPilot
license: MIT
contents:
  - src: $STAGE_DIR/etcdpilot
    dst: /usr/bin/etcdpilot
    file_info:
      mode: 0755
  - src: $STAGE_DIR/web/dist
    dst: /usr/share/etcdpilot/web/dist
  - src: $STAGE_DIR/config/config.prod.toml
    dst: /etc/etcdpilot/config.toml
    type: config|noreplace
  - src: $STAGE_DIR/config/config.test.toml
    dst: /usr/share/etcdpilot/config/config.test.toml
  - src: $STAGE_DIR/RUNNING.md
    dst: /usr/share/doc/etcdpilot/RUNNING.md
  - src: $STAGE_DIR/README.md
    dst: /usr/share/doc/etcdpilot/README.md
  - src: $STAGE_DIR/LICENSE
    dst: /usr/share/doc/etcdpilot/LICENSE
  - src: $SCRIPT_DIR/etcdpilot.service
    dst: /usr/lib/systemd/system/etcdpilot.service
  - dst: /var/lib/etcdpilot
    type: dir
scripts:
  postinstall: $SCRIPT_DIR/after-install.sh
  postremove: $SCRIPT_DIR/after-remove.sh
EOF

nfpm package --config "$CONFIG" --packager rpm --target "$OUT_DIR/etcdpilot_${VERSION#v}_${RUNTIME}.rpm"
