#!/usr/bin/env bash
# Deploy the permanent ProcedWorld game server on the reference VPS.
# Builds pw-server from the current checkout in the Rust image, packs it into a small Debian image,
# and (re)starts the `pw-server` container: loopback port 8100 only (PORTS.md), on the `proxy`
# network so Caddy can reach it as pw-server:8100, worlds persisted in /opt/stacks/procedworld/server/data.
# Caddy (/opt/infra/caddy/Caddyfile) routes /ws and /health to it behind the site's HTTP Basic auth.
#
# Usage: VPS_SSH=deploy@<VPS_HOST> infra/scripts/deploy-server.sh
set -uo pipefail
root=$(git rev-parse --show-toplevel)
target=${VPS_SSH:?set VPS_SSH, e.g. deploy@<VPS_HOST>}
rev=$(git -C "$root" rev-parse --short HEAD)
work="/home/deploy/pw-build/deploy"
stack="/opt/stacks/procedworld/server"

echo "deploying pw-server at $rev..."
tar -C "$root" --exclude='target' --exclude='.git' -czf - engine data | \
  ssh -o BatchMode=yes "$target" "set -e; rm -rf $work && mkdir -p $work && tar xzf - -C $work
    flock /tmp/pw-cargo-target.lock docker run --rm -v $work:/src -w /src/engine \
      -v pw-cargo-registry:/usr/local/cargo/registry -v pw-cargo-target:/target -e CARGO_TARGET_DIR=/target \
      rust:1-slim sh -c 'cargo build --release -q -p pw-server && cp /target/release/pw-server /src/pw-server && chown -R 1000:1000 /src'
    printf 'FROM debian:bookworm-slim\nCOPY pw-server /usr/local/bin/pw-server\nENTRYPOINT [\"/usr/local/bin/pw-server\"]\n' > $work/Dockerfile
    docker build -q -t pw-server:$rev -t pw-server:latest $work >/dev/null
    mkdir -p $stack/data
    docker rm -f pw-server >/dev/null 2>&1 || true
    docker run -d --name pw-server --restart unless-stopped --network proxy \
      -p 127.0.0.1:8100:8100 -e PW_SERVER_BIND=0.0.0.0:8100 -e PW_DATA_DIR=/data \
      -v $stack/data:/data --label procedworld.rev=$rev pw-server:latest >/dev/null
    for i in 1 2 3 4 5 6 7 8 9 10; do curl -fsS 127.0.0.1:8100/health && break; sleep 1; done; echo" \
  && echo "pw-server $rev is up" || { echo "deploy failed"; exit 1; }
