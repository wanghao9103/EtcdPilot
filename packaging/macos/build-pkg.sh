#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-dev}"
RUNTIME="${2:-macos-arm64}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STAGE_DIR="$REPO_ROOT/build/package/EtcdPilot-$VERSION-$RUNTIME"
ROOT_DIR="$REPO_ROOT/build/pkgroot-$RUNTIME"
COMPONENT_PKG="$REPO_ROOT/build/EtcdPilot-$VERSION-$RUNTIME-component.pkg"
OUT_DIR="$REPO_ROOT/artifacts"
OUT_PKG="$OUT_DIR/EtcdPilot-$VERSION-$RUNTIME.pkg"
IDENTIFIER="com.etcdpilot.app"

if [ ! -d "$STAGE_DIR" ]; then
  echo "Portable package stage directory not found: $STAGE_DIR" >&2
  exit 1
fi

rm -rf "$ROOT_DIR" "$COMPONENT_PKG"
mkdir -p "$ROOT_DIR/usr/local/etcdpilot" "$OUT_DIR"
cp -R "$STAGE_DIR/." "$ROOT_DIR/usr/local/etcdpilot/"

pkgbuild \
  --root "$ROOT_DIR" \
  --identifier "$IDENTIFIER" \
  --version "${VERSION#v}" \
  --install-location "/" \
  "$COMPONENT_PKG"

productbuild \
  --package "$COMPONENT_PKG" \
  "$OUT_PKG"

echo "Created package: $OUT_PKG"
