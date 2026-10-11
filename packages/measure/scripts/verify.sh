#!/usr/bin/env sh
set -eu

PHALCOM_BIN="${PHALCOM_BIN:-phalcom}"
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

"$PHALCOM_BIN" "$ROOT"
"$PHALCOM_BIN" "$ROOT/tests/runtime"
"$PHALCOM_BIN" "$ROOT/tests/projects/custom-units"
"$PHALCOM_BIN" "$ROOT/tests/projects/custom-units-consumer"

for file in "$ROOT"/tests/compile/pass/*.ph; do
  "$PHALCOM_BIN" "$file"
done

for file in "$ROOT"/tests/compile/fail/*.ph; do
  if "$PHALCOM_BIN" "$file" >/dev/null 2>&1; then
    echo "expected compile failure but program succeeded: $file" >&2
    exit 1
  fi
done

echo "measure verification complete"
