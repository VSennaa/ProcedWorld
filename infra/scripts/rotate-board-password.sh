#!/usr/bin/env bash
# Rotates the HTTP Basic password of the agent board (run on the VPS as deploy, once per sprint).
# Prints the new password once; only its bcrypt hash is stored (in /opt/infra/caddy/Caddyfile).
# Usage: infra/scripts/rotate-board-password.sh [user]   (default user: procedworld)
set -euo pipefail
user=${1:-procedworld}
file=/opt/infra/caddy/Caddyfile
pass=$(head -c 24 /dev/urandom | base64 | tr -d '/+=' | head -c 20)
hash=$(docker run --rm caddy:2-alpine caddy hash-password --plaintext "$pass" | tail -1)
# Replace the hash on the "<user> <hash>" line inside basic_auth.
sed -i -E "s|^([[:space:]]*${user}[[:space:]]+).*|\\1${hash}|" "$file"
docker exec caddy caddy reload --config /etc/caddy/Caddyfile >/dev/null
echo "board user: ${user}"
echo "board password (save it now, it is not stored): ${pass}"
