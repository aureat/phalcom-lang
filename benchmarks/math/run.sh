#!/usr/bin/env bash
#
# run.sh — execute every math benchmark and assert its self-verification passed.
#
# A benchmark PASSES iff it runs with exit status 0 and every non-blank line of
# stdout is exactly `true` (the programs print one boolean per asserted identity).
#
# Files whose header says "Tier 2" depend on language features that have not
# landed yet (var, collections, stdlib surface). They are expected to fail today,
# so a failure there is reported as PENDING and does NOT fail the run. Pass
# --strict to treat PENDING as failure once those phases are in.
#
# Usage:
#   benchmarks/math/run.sh            # run all; nonzero exit iff a ready file fails
#   benchmarks/math/run.sh --strict   # also require Tier-2 files to pass
#   benchmarks/math/run.sh a.ph b.ph  # run only the named files
#
set -u

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo" || exit 2

strict=0
files=()
for arg in "$@"; do
  case "$arg" in
    --strict) strict=1 ;;
    *)        files+=("$arg") ;;
  esac
done
if [ "${#files[@]}" -eq 0 ]; then
  # default: every .ph in this directory, sorted
  while IFS= read -r f; do files+=("$f"); done < <(cd "$here" && ls -1 *.ph | sort)
fi

echo "==> building phalcom CLI"
if ! cargo build -q -p phalcom-core --bin phalcom; then
  echo "BUILD FAILED — the tree does not compile; cannot run benchmarks." >&2
  exit 2
fi
bin="$repo/target/debug/phalcom"

pass=0 fail=0 pending=0 rc=0

for name in "${files[@]}"; do
  path="$here/$name"
  [ -f "$path" ] || { echo "MISSING  $name" >&2; fail=$((fail+1)); rc=1; continue; }

  tier2=0
  grep -qi 'Tier 2' "$path" && tier2=1

  out="$("$bin" "$path" 2>&1)"
  status=$?

  # bad = any non-blank stdout line that is not exactly "true"
  bad="$(printf '%s\n' "$out" | grep -vE '^\s*(true)?\s*$' || true)"

  if [ "$status" -eq 0 ] && [ -z "$bad" ] && [ -n "$out" ]; then
    echo "PASS     $name"
    pass=$((pass+1))
  elif [ "$tier2" -eq 1 ] && [ "$strict" -eq 0 ]; then
    echo "PENDING  $name (Tier 2 — feature not yet implemented)"
    pending=$((pending+1))
  else
    echo "FAIL     $name"
    printf '%s\n' "$out" | sed 's/^/           | /'
    fail=$((fail+1))
    rc=1
  fi
done

echo "---"
echo "pass=$pass fail=$fail pending=$pending"
exit "$rc"
