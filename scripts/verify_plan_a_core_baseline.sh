#!/usr/bin/env bash
set -u

# Compare the exact phalcom-core failure set against the Plan-A entry baseline
# without touching the caller's worktree. Each integration target is run on
# both revisions so PASS, FAIL, and TIMEOUT/HANG remain distinguishable.

BASELINE_COMMIT="${PLAN_A_CORE_BASELINE:-d60e4589352ac5f4167ba295e7e2a5f6c870ef4b}"
TIMEOUT_SECONDS="${PLAN_A_CORE_TIMEOUT_SECONDS:-600}"
TEST_TIMEOUT_SECONDS="${PLAN_A_CORE_TEST_TIMEOUT_SECONDS:-120}"
TEST_THREADS_ARG="${PLAN_A_CORE_TEST_THREADS:-}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/plan-a-core-baseline.XXXXXX")"
BASELINE_ROOT="$TEMP_ROOT/baseline"
BASELINE_TARGET="$TEMP_ROOT/baseline-target"
CURRENT_TARGET="$TEMP_ROOT/current-target"

cleanup() {
    git -C "$REPO_ROOT" worktree remove --force "$BASELINE_ROOT" >/dev/null 2>&1 || true
    rm -rf "$TEMP_ROOT"
}
if [[ -n "${PLAN_A_CORE_KEEP_TEMP:-}" ]]; then
    trap 'echo "temporary comparison root: $TEMP_ROOT"' EXIT
else
    trap cleanup EXIT
fi

if ! git -C "$REPO_ROOT" cat-file -e "$BASELINE_COMMIT^{commit}"; then
    echo "baseline commit is unavailable: $BASELINE_COMMIT" >&2
    exit 2
fi

git -C "$REPO_ROOT" worktree add --detach "$BASELINE_ROOT" "$BASELINE_COMMIT" >/dev/null

TARGETS="$(
    cd "$REPO_ROOT" || exit 1
    cargo metadata --no-deps --format-version 1 |
        python3 -c 'import json, sys; data=json.load(sys.stdin); package=next(p for p in data["packages"] if p["name"] == "phalcom-core"); print("\n".join(target["name"] for target in package["targets"] if "test" in target["kind"]))'
)"

run_with_timeout() {
    python3 - "$TIMEOUT_SECONDS" "$@" <<'PY'
import os
import signal
import subprocess
import sys

timeout_seconds = float(sys.argv[1])
command = sys.argv[2:]
process = subprocess.Popen(command, start_new_session=True)
try:
    process.wait(timeout=timeout_seconds)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    raise SystemExit(124)
raise SystemExit(process.returncode)
PY
}

run_target() {
    local root="$1"
    local target="$2"
    local cargo_target="$3"
    local output="$4"
    local status_file="$5"
    local failures_file="$6"
    local rc

    mkdir -p "$cargo_target"
    if [[ -n "$TEST_THREADS_ARG" ]]; then
        if (
            cd "$root" || exit 1
            RUSTFLAGS='' CARGO_TARGET_DIR="$cargo_target" run_with_timeout cargo test -p phalcom-core --test "$target" -- "--test-threads=$TEST_THREADS_ARG"
        ) >"$output" 2>&1; then
            rc=0
        else
            rc=$?
        fi
    elif (
        cd "$root" || exit 1
        RUSTFLAGS='' CARGO_TARGET_DIR="$cargo_target" run_with_timeout cargo test -p phalcom-core --test "$target" --
    ) >"$output" 2>&1; then
        rc=0
    else
        rc=$?
    fi

    case "$rc" in
        124|137|142) echo "TIMEOUT/HANG" >"$status_file" ;;
        0) echo "PASS" >"$status_file" ;;
        *) echo "FAIL" >"$status_file" ;;
    esac

    awk '$1 == "test" && $(NF) == "FAILED" { sub(/^test /, ""); sub(/ \.\.\. FAILED$/, ""); print }' "$output" |
        LC_ALL=C sort -u >"$failures_file"
}

enumerate_target_tests() {
    local root="$1"
    local target="$2"
    local cargo_target="$3"
    local failures_file="$4"
    local timeouts_file="$5"
    local names
    local binary

    binary="$(find "$cargo_target/debug" -type f -perm -111 -name "$target-*" -print -quit 2>/dev/null)"
    if [[ -z "$binary" ]]; then
        (
            cd "$root" || exit 1
            RUSTFLAGS='' CARGO_TARGET_DIR="$cargo_target" cargo test -p phalcom-core --test "$target" --no-run
        ) >/dev/null 2>&1
        binary="$(find "$cargo_target/debug" -type f -perm -111 -name "$target-*" -print -quit 2>/dev/null)"
    fi
    if [[ -z "$binary" ]]; then
        echo "could not locate test executable for target $target" >&2
        return 2
    fi

    names="$("$binary" --list 2>/dev/null | sed -n 's/: test$//p')"
    : >"$failures_file"
    : >"$timeouts_file"
    local results_file="${failures_file}.results"
    printf '%s\n' "$names" |
        xargs -P "${PLAN_A_CORE_PARALLELISM:-8}" -n 1 bash -c '
            root="$1"
            binary="$2"
            timeout_seconds="$3"
            test_name="$4"
            (
                cd "$root" || exit 1
                python3 - "$timeout_seconds" "$binary" "$test_name" <<'PY'
import os
import signal
import subprocess
import sys

timeout_seconds = float(sys.argv[1])
process = subprocess.Popen(sys.argv[2:] + ["--exact", "--test-threads=1"], start_new_session=True)
try:
    process.wait(timeout=timeout_seconds)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
    raise SystemExit(124)
raise SystemExit(process.returncode)
PY
            ) >/dev/null 2>&1
            rc=$?
            case "$rc" in
                124|137|142) echo "TIMEOUT $test_name" ;;
                0) ;;
                *) echo "FAIL $test_name" ;;
            esac
        ' _ "$root" "$binary" "$TEST_TIMEOUT_SECONDS" >"$results_file"
    sed -n 's/^FAIL //p' "$results_file" >"$failures_file"
    sed -n 's/^TIMEOUT //p' "$results_file" >"$timeouts_file"
    rm -f "$results_file"
    LC_ALL=C sort -u -o "$failures_file" "$failures_file"
    LC_ALL=C sort -u -o "$timeouts_file" "$timeouts_file"
}

declare -a BASELINE_FAILURE_FILES=()
declare -a CURRENT_FAILURE_FILES=()
declare -a TARGET_NAMES=()

echo "Plan-A phalcom-core baseline comparison"
echo "baseline commit: $BASELINE_COMMIT"
echo "targets:"
for target in $TARGETS; do
    TARGET_NAMES+=("$target")
    baseline_output="$TEMP_ROOT/baseline-$target.log"
    current_output="$TEMP_ROOT/current-$target.log"
    baseline_status="$TEMP_ROOT/baseline-$target.status"
    current_status="$TEMP_ROOT/current-$target.status"
    baseline_failures="$TEMP_ROOT/baseline-$target.failures"
    current_failures="$TEMP_ROOT/current-$target.failures"
    baseline_timeouts="$TEMP_ROOT/baseline-$target.timeouts"
    current_timeouts="$TEMP_ROOT/current-$target.timeouts"
    BASELINE_FAILURE_FILES+=("$baseline_failures")
    CURRENT_FAILURE_FILES+=("$current_failures")

    run_target "$BASELINE_ROOT" "$target" "$BASELINE_TARGET" "$baseline_output" "$baseline_status" "$baseline_failures"
    run_target "$REPO_ROOT" "$target" "$CURRENT_TARGET" "$current_output" "$current_status" "$current_failures"

    if [[ "$(cat "$baseline_status")" == "TIMEOUT/HANG" ]]; then
        enumerate_target_tests "$BASELINE_ROOT" "$target" "$BASELINE_TARGET" "$baseline_failures" "$baseline_timeouts"
    else
        : >"$baseline_timeouts"
    fi
    if [[ "$(cat "$current_status")" == "TIMEOUT/HANG" ]]; then
        enumerate_target_tests "$REPO_ROOT" "$target" "$CURRENT_TARGET" "$current_failures" "$current_timeouts"
    else
        : >"$current_timeouts"
    fi

    echo "  $target: baseline=$(cat "$baseline_status"), current=$(cat "$current_status")"
    echo "    baseline failing names: $(paste -sd, "$baseline_failures" | sed 's/^$/<none>/')"
    echo "    current failing names: $(paste -sd, "$current_failures" | sed 's/^$/<none>/')"
    echo "    baseline timeout/hang names: $(paste -sd, "$baseline_timeouts" | sed 's/^$/<none>/')"
    echo "    current timeout/hang names: $(paste -sd, "$current_timeouts" | sed 's/^$/<none>/')"
done

cat "${BASELINE_FAILURE_FILES[@]}" 2>/dev/null | LC_ALL=C sort -u >"$TEMP_ROOT/baseline.failures" || :
cat "${CURRENT_FAILURE_FILES[@]}" 2>/dev/null | LC_ALL=C sort -u >"$TEMP_ROOT/current.failures" || :

echo "current failures minus baseline failures:"
CURRENT_ONLY="$(comm -23 "$TEMP_ROOT/current.failures" "$TEMP_ROOT/baseline.failures")"
if [[ -n "$CURRENT_ONLY" ]]; then
    echo "$CURRENT_ONLY"
else
    echo "<none>"
fi

status_regressions=0
echo "status regressions:"
for target in "${TARGET_NAMES[@]}"; do
    baseline_status="$(cat "$TEMP_ROOT/baseline-$target.status")"
    current_status="$(cat "$TEMP_ROOT/current-$target.status")"
    if [[ "$current_status" == "FAIL" && "$baseline_status" == "PASS" ]] ||
        [[ "$current_status" == "TIMEOUT/HANG" && "$baseline_status" != "TIMEOUT/HANG" ]]; then
        echo "  $target: baseline=$baseline_status, current=$current_status"
        status_regressions=1
    fi
done
if [[ "$status_regressions" -eq 0 ]]; then
    echo "<none>"
fi

if [[ -n "$CURRENT_ONLY" || "$status_regressions" -ne 0 ]]; then
    echo "RESULT: FAIL"
    exit 1
fi

echo "RESULT: PASS"
