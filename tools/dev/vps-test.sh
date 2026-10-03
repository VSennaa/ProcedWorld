#!/usr/bin/env bash
# Build and test the Rust engine on the reference VPS inside the official Rust Docker image.
# Why: on the developer PC, Windows Smart App Control blocks freshly compiled executables
# (build scripts and test binaries), and the Codex sandbox has no network.
#
# Usage: tools/dev/vps-test.sh [repo-root] [cargo args...]   (default args: test --workspace)
# Env:   VPS (ssh target, default from ~/.ssh/config alias "procedworld-vps" or $VPS_SSH)
# Syncs engine/ and data/ (no target/), runs cargo in Docker with cached registry and target volumes.
# The cargo target volume is shared: builds are serialized with flock on the host, otherwise two
# concurrent runs can execute each other's freshly built test binaries.
set -uo pipefail
root=${1:-$(git rev-parse --show-toplevel)}; shift || true
args=("$@"); [ ${#args[@]} -eq 0 ] && args=(test --workspace)
target=${VPS_SSH:?set VPS_SSH, e.g. deploy@<VPS_HOST>}
work="/home/deploy/pw-build/$(basename "$root")"

tar -C "$root" --exclude='target' --exclude='.git' -czf - engine data 2>/dev/null | \
  ssh -o BatchMode=yes "$target" "rm -rf '$work' && mkdir -p '$work' && tar xzf - -C '$work' && \
    flock /tmp/pw-cargo-target.lock docker run --rm -v '$work':/src -w /src/engine \
      -v pw-cargo-registry:/usr/local/cargo/registry -v pw-cargo-target:/target \
      -e CARGO_TARGET_DIR=/target -e CARGO_TERM_COLOR=never rust:1-slim \
      sh -c 'cargo ${args[*]} -j 2 2>&1'; echo \"VPS_EXIT=\$?\""
