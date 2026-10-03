#!/usr/bin/env bash
# Run the simulation harness on the reference VPS (Docker), from the current checkout.
# Why: Windows Smart App Control blocks freshly compiled Rust binaries on the developer PC.
#
# Usage: VPS_SSH=deploy@<VPS_HOST> tools/dev/vps-harness.sh [harness args...]
#   default args: run --seed 20261001 --civs 8 --turns 300
#   examples:     tools/dev/vps-harness.sh run --seed 42 --civs 4 --turns 1000
#                 tools/dev/vps-harness.sh replay out/42
# The cargo target volume is shared: builds are serialized with flock on the host, otherwise two
# concurrent runs can execute each other's freshly built test binaries.
set -uo pipefail
root=$(git rev-parse --show-toplevel)
target=${VPS_SSH:?set VPS_SSH, e.g. deploy@<VPS_HOST>}
args="${*:-run --seed 20261001 --civs 8 --turns 300}"
work="/home/deploy/pw-build/harness"

tar -C "$root" --exclude='target' --exclude='.git' -czf - engine data | \
  ssh -o BatchMode=yes "$target" "rm -rf $work && mkdir -p $work && tar xzf - -C $work && \
    flock /tmp/pw-cargo-target.lock docker run --rm -v $work:/src -w /src/engine -v pw-cargo-registry:/usr/local/cargo/registry \
      -v pw-cargo-target:/target -e CARGO_TARGET_DIR=/target rust:1-slim sh -c \
      'cargo build --release -q -p pw-harness 2>&1 | tail -3; cd /src && /target/release/pw-harness $args; chown -R 1000:1000 /src'"
