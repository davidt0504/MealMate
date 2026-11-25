#!/usr/bin/env bash
set -euo pipefail

if [[ ${1-} == --help || ${1-} == -h ]]; then
  printf '%s\n' 'Usage: bash tools/run_tests.sh [FLUTTER_TEST_ARGS...]' \
    'Run Rust workspace tests, build the host release library, then run Flutter tests.' \
    'Additional arguments go to Flutter from the repository root.'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$root/rust"
host=$(rustc -vV | awk '$1 == "host:" { print $2 }')
if [[ -z "$host" ]]; then
  printf '%s\n' 'Could not determine the Rust host target.' >&2
  exit 1
fi
target_dir="$root/rust/target"
native_dir="$target_dir/$host/release"
native_file=$(rustc --target "$host" --print file-names --crate-name kimatta_bridge --crate-type cdylib - </dev/null)

cargo test --workspace --target "$host" --target-dir "$target_dir"
cargo build --release --target "$host" --target-dir "$target_dir"
if [[ ! -f "$native_dir/$native_file" ]]; then
  printf 'Native bridge library missing at %s\n' "$native_dir/$native_file" >&2
  exit 1
fi
cd -- "$root"
export FRB_DART_LOAD_EXTERNAL_LIBRARY_NATIVE_LIB_DIR="$native_dir"
exec flutter test "$@"
