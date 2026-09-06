#!/usr/bin/env bash
# The gate, in one place. CI runs this file and nothing else, so what runs on the machine and what
# runs before a commit cannot drift.
#
# Why a script rather than a list of steps: on 2026-09-06 `verify` printed "7 checks ran, 0 failed"
# while clippy was reporting a finding, and the commit went out green-looking. `verify` is not the
# gate — it is one stage of it. A gate that is a habit is a gate that gets skipped; this one exits
# non-zero, so it is the exit code that has to be argued with.
#
# Rule: seconds, and green before every commit. Budget is 8 minutes on a cold CI cache (verify.yml).

set -euo pipefail

cd "$(dirname "$0")"

# --locked in CI, unlocked locally: a lockfile update mid-milestone is a decision, not a side effect,
# but a developer adding a dependency should not have to fight the gate to see it compile.
LOCKED=${AURORA_GATE_LOCKED:-}

stage() { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }

stage "format"
cargo fmt --all -- --check

stage "build"
cargo build --workspace $LOCKED

stage "clippy"
# -D warnings is the whole point: pedantic findings return 0 on their own, which is how a red tree
# reads as green.
cargo clippy --workspace --all-targets $LOCKED -- -D warnings

stage "test"
cargo test --workspace $LOCKED

stage "checks"
# The rules the compiler cannot hold. Each prints its rule inventory and its exemption count, and
# the count must read zero — §17's empty exemption list is only a claim if something counts it.
cargo run -q -p aurora-tools $LOCKED -- verify

printf '\n\033[1;32mgate: green\033[0m\n'
