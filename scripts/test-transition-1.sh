#!/usr/bin/env bash
# Focused Transition 1 test runner.
#
# Keeps legacy corpus checks and new-syntax probes separate during the
# declaration-syntax flag day.  It deliberately builds only the `phalcom`
# binary; it never asks Cargo to build or run every integration-test target.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

usage() {
  cat <<'USAGE'
Usage:
  scripts/test-transition-1.sh legacy <label/case> [--negative]
  scripts/test-transition-1.sh probe <name> [--negative]
  scripts/test-transition-1.sh rust <ast|core|lsp>

legacy <label/case>
    Run one unchanged case from phalcom-core/tests/lang/.  Example:
      scripts/test-transition-1.sh legacy classes/class_labeled_arg_method_definition

probe <name>
    Run one new-syntax case from phalcom-core/tests/transition-1/.  Example:
      scripts/test-transition-1.sh probe task-02/setter_and_subscript

--negative
    Treat the matching .expected file as a required diagnostic substring.
    Use for compile-error, syntax-error, and runtime-error cases.

rust <crate>
    Run only in-crate tests whose name starts `transition_1_`.  This refuses
    to pass when the selected crate has no such tests, preventing an empty
    Cargo filter from being reported as coverage.
USAGE
}

fail() {
  printf 'transition-1 test: %s\n' "$*" >&2
  exit 2
}

normalize_case() {
  local root="$1"
  local name="$2"
  local path="$root/$name"
  path="${path%.ph}"
  printf '%s.ph\n' "$path"
}

run_case() {
  local case_path="$1"
  local negative="$2"
  local expected="${case_path%.ph}.expected"

  [[ -f "$case_path" ]] || fail "case not found: $case_path"
  [[ -f "$expected" ]] || fail "expected output not found: $expected"

  cargo build -q -p phalcom-core --bin phalcom

  local out err status
  out="$(mktemp -t phalcom-transition-1-out.XXXXXX)"
  err="$(mktemp -t phalcom-transition-1-err.XXXXXX)"
  trap 'rm -f "$out" "$err"' RETURN

  local -a flags=()
  local flag_line
  flag_line="$(sed -n 's/^[[:space:]]*\/\/ flags:[[:space:]]*//p' "$case_path" | head -n 1)"
  if [[ -n "$flag_line" ]]; then
    read -r -a flags <<<"$flag_line"
  fi

  set +e
  if [[ ${#flags[@]} -gt 0 ]]; then
    target/debug/phalcom "${flags[@]}" "$case_path" >"$out" 2>"$err"
  else
    target/debug/phalcom "$case_path" >"$out" 2>"$err"
  fi
  status=$?
  set -e

  if [[ "$negative" == true ]]; then
    [[ "$status" -ne 0 ]] || fail "$case_path unexpectedly succeeded"
    local note
    note="$(<"$expected")"
    [[ -n "$note" ]] || fail "negative expected file is empty: $expected"
    rg -F --quiet -- "$note" "$out" "$err" || {
      printf '%s did not contain expected diagnostic:\n%s\n' "$case_path" "$note" >&2
      cat "$out" "$err" >&2
      exit 1
    }
  else
    [[ "$status" -eq 0 ]] || {
      if rg -F --quiet 'core module (core.ph) must compile and run cleanly' "$err"; then
        printf 'bootstrap blocked before %s; core.ph must accept the current parser before source-case evidence is available:\n' "$case_path" >&2
      fi
      printf '%s failed (exit %s):\n' "$case_path" "$status" >&2
      cat "$err" >&2
      exit 1
    }
    diff -u "$expected" "$out"
  fi

  printf 'PASS %s\n' "$case_path"
}

run_rust() {
  local crate="$1"
  case "$crate" in
    ast) crate="phalcom-ast" ;;
    core) crate="phalcom-core" ;;
    lsp) crate="phalcom-lsp" ;;
    *) fail "rust crate must be ast, core, or lsp" ;;
  esac

  local listed
  listed="$(cargo test -q -p "$crate" --lib -- --list | rg '^transition_1_' || true)"
  [[ -n "$listed" ]] || fail "no transition_1_ tests in $crate; add a focused new-syntax unit test first"
  cargo test -p "$crate" --lib transition_1_
}

[[ $# -ge 1 ]] || { usage >&2; exit 2; }

case "$1" in
  -h|--help)
    usage
    ;;
  legacy|probe)
    [[ $# -ge 2 ]] || fail "missing case name"
    mode="$1"
    name="$2"
    shift 2
    negative=false
    if [[ $# -gt 0 ]]; then
      [[ $# -eq 1 && "$1" == --negative ]] || { usage >&2; exit 2; }
      negative=true
    fi
    if [[ "$mode" == legacy ]]; then
      run_case "$(normalize_case phalcom-core/tests/lang "$name")" "$negative"
    else
      run_case "$(normalize_case phalcom-core/tests/transition-1 "$name")" "$negative"
    fi
    ;;
  rust)
    [[ $# -eq 2 ]] || { usage >&2; exit 2; }
    run_rust "$2"
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
