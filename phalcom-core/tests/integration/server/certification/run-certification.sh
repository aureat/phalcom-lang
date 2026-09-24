#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ -z "${PHALCOM_GOLDEN_DRIVER:-}" ]; then
    echo "PHALCOM_GOLDEN_DRIVER is required." >&2
    echo "See certification/driver-contract.md for the adapter contract." >&2
    exit 2
fi

run() {
    printf '%s\n' "==> $1"
    "$PHALCOM_GOLDEN_DRIVER" "$1" "$ROOT"
}

run clean-build
run persistence-build
run incremental-noop
run incremental-leaf
run incremental-implementation
run incremental-signature
run negative-corpus

printf '%s\n' "PHALCOM GOLDEN CERTIFICATION: PASS"
