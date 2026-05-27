#!/usr/bin/env bash
# Run all integration test cases against the docker image built from the nix flake.
# All tests run inside a single container to avoid per-test startup overhead.
#
# Usage:
#   ./dev/ci/docker_integration_tests.sh [--no-build] [--filter PATTERN]
#
# Options:
#   --no-build     Skip nix build / docker load (use already-loaded image)
#   --filter PAT   Only run test cases whose path matches PAT (grep -F)
#
# The image runs as nobody by default.  test_cases/ is mounted read-only.
# The verify command uses --output-dir /test_cases_work/<rel> so its working
# tree has the same path depth as the real test_cases tree.  This matters for
# test cases whose toml references input files via relative paths that escape
# the test case directory (e.g. '../../../../sample_data/…'): verify.rs calls
# create_dir_all on temp_path.join(that_relative_path), which traverses '..'
# literally.  With a shallow /tmp temp dir the traversal exits the tmpfs and
# hits the read-only container root.  Anchoring the output dir at the right
# depth keeps the escaped paths inside the writable /test_cases_work tmpfs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

IMAGE="fastqrab:latest"
DO_BUILD=1
FILTER=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-build) DO_BUILD=0; shift ;;
        --filter) FILTER="$2"; shift 2 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

cd "$PROJECT_ROOT"

# ---------------------------------------------------------------------------
# Build and load the docker image
# ---------------------------------------------------------------------------
if [[ $DO_BUILD -eq 1 ]]; then
    echo "==> Building docker image (nix build .#fastqrab-docker)..."
    nix build .#fastqrab-docker

    echo "==> Loading image into docker..."
    docker load < ./result
fi

# ---------------------------------------------------------------------------
# Run all tests inside a single container
# ---------------------------------------------------------------------------
TEST_CASES_DIR="$PROJECT_ROOT/test_cases"

# Script executed inside the container.  Outputs structured lines:
#   PASS <secs> <rel_path>
#   FAIL <secs> <rel_path>
#   ERR  <text>            (stderr lines for the preceding FAIL)
#
# Single-quoted so $-signs are passed literally to the container's bash.
# shellcheck disable=SC2016
INNER_SCRIPT='
set -euo pipefail
while IFS= read -r -d "" f; do
    dir="$(dirname "$f")"
    case "$(basename "$dir")" in actual*) continue ;; esac
    # Skip tests that require the allocation-counting binary variant
    if [ -f "$dir/measure_alloc" ]; then
        continue
    fi
    # When a test directory has test.sh, only run input.toml (not input2.toml etc.)
    if [ -f "$dir/test.sh" ] && [ "$(basename "$f")" != "input.toml" ]; then
        continue
    fi
    rel="${f#/test_cases/}"
    if [ -n "${FILTER_PAT:-}" ] && ! echo "$rel" | grep -qF "$FILTER_PAT"; then
        continue
    fi
    t=$SECONDS
    rel_dir="$(dirname "$rel")"
    exit_code=0
    fastqrab verify "$f" --output-dir "/test_cases_work/$rel_dir/actual_docker" \
            --unsafe-call-prep-sh >/tmp/t_out 2>/tmp/t_err || exit_code=$?
    if [ -f "$dir/should_panic" ]; then
        # Test is expected to fail; pass iff the command exited non-zero
        if [ $exit_code -ne 0 ]; then
            printf "PASS %d %s\n" "$((SECONDS - t))" "$rel"
        else
            printf "FAIL %d %s\n" "$((SECONDS - t))" "$rel"
            printf "ERR  expected non-zero exit but command succeeded\n"
        fi
    else
        if [ $exit_code -eq 0 ]; then
            printf "PASS %d %s\n" "$((SECONDS - t))" "$rel"
        else
            printf "FAIL %d %s\n" "$((SECONDS - t))" "$rel"
            sed "s/^/ERR /" /tmp/t_err
        fi
    fi
done < <(find /test_cases -name "input*.toml" -print0 | sort -z)
'

PASS=0
FAIL=0
declare -a FAILURES=()
TIMING_LOG=""

while IFS= read -r line; do
    case "$line" in
        PASS\ *)
            read -r _ secs rel_toml <<< "$line"
            printf "  %-70s PASS  %3ds\n" "$rel_toml" "$secs"
            PASS=$((PASS + 1))
            TIMING_LOG="${TIMING_LOG}${secs} ${rel_toml}"$'\n'
            ;;
        FAIL\ *)
            read -r _ secs rel_toml <<< "$line"
            printf "  %-70s FAIL  %3ds\n" "$rel_toml" "$secs"
            FAIL=$((FAIL + 1))
            FAILURES+=("$rel_toml")
            TIMING_LOG="${TIMING_LOG}${secs} ${rel_toml}"$'\n'
            ;;
        ERR\ *)
            echo "    ${line#ERR }"
            ;;
    esac
done < <(docker run --rm \
    -v "$TEST_CASES_DIR:/test_cases:ro" \
    -v "$PROJECT_ROOT/cookbooks:/cookbooks:ro" \
    --tmpfs /tmp:mode=1777 \
    --tmpfs /test_cases_work:mode=1777 \
    -e FILTER_PAT="${FILTER}" \
    --entrypoint /bin/bash \
    "$IMAGE" \
    -c "$INNER_SCRIPT")

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "========================================"
echo "Results: $PASS passed, $FAIL failed"
echo "========================================"

echo ""
echo "Slowest 10 tests:"
echo "$TIMING_LOG" | sort -rn | head -10 | while read -r secs name; do
    printf "  %3ds  %s\n" "$secs" "$name"
done

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    echo ""
    echo "Failed test cases:"
    for f in "${FAILURES[@]}"; do
        echo "  - $f"
    done
    exit 1
fi
