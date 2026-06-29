#!/usr/bin/env sh
set -eu

if ! id etcdpilot >/dev/null 2>&1; then
  useradd --system --home /var/lib/etcdpilot --shell /sbin/nologin etcdpilot 2>/dev/null || \
    useradd --system --home /var/lib/etcdpilot --shell /usr/sbin/nologin etcdpilot
fi

mkdir -p /var/lib/etcdpilot
chown -R etcdpilot:etcdpilot /var/lib/etcdpilot || true

if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
fi
