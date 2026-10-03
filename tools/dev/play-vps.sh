#!/usr/bin/env bash
# Play the Windows client against a throwaway pw-server on the VPS, through an SSH tunnel.
# Builds pw-server from the current checkout, starts it loopback-only on the VPS (no public port),
# forwards local 127.0.0.1:8100, and stops both the tunnel and the container on Ctrl+C.
#
# Usage: VPS_SSH=deploy@<VPS_HOST> tools/dev/play-vps.sh
# Then open build/windows/ProcedWorld.exe and keep the default URL ws://127.0.0.1:8100/ws.
set -uo pipefail
root=$(git rev-parse --show-toplevel)
target=${VPS_SSH:?set VPS_SSH, e.g. deploy@<VPS_HOST>}
work="/home/deploy/pw-build/play"
name="pw-server-play"

echo "building pw-server on the VPS..."
tar -C "$root" --exclude='target' --exclude='.git' -czf - engine data | \
  ssh -o BatchMode=yes "$target" "docker rm -f $name >/dev/null 2>&1; rm -rf $work && mkdir -p $work && tar xzf - -C $work && \
    flock /tmp/pw-cargo-target.lock docker run --rm -v $work:/src -w /src/engine -v pw-cargo-registry:/usr/local/cargo/registry \
      -v pw-cargo-target:/target -e CARGO_TARGET_DIR=/target rust:1-slim sh -c \
      'cargo build --release -q -p pw-server 2>&1 | tail -3; cp /target/release/pw-server /src/pw-server; chown -R 1000:1000 /src' && \
    docker run -d --rm --name $name -p 127.0.0.1:8100:8100 -e PW_SERVER_BIND=0.0.0.0:8100 -v $work:/src rust:1-slim /src/pw-server >/dev/null" \
  || { echo "failed to start the server"; exit 1; }

cleanup() { ssh -o BatchMode=yes "$target" "docker rm -f $name >/dev/null 2>&1"; echo "server stopped"; }
trap cleanup EXIT
echo "server up. Tunnel open on ws://127.0.0.1:8100/ws - press Ctrl+C to stop."
ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -N -L 8100:127.0.0.1:8100 "$target"
