#!/usr/bin/env bash
# Run all integration test cases against the docker image built from the nix flake.
#
# Usage:
#   ./dev/ci/docker_integration_tests.sh [--no-build] [--filter PATTERN]
#
# Options:
#   --no-build     Skip nix build / docker load (use already-loaded image)
#   --filter PAT   Only run test cases whose path matches PAT (grep -F)
#
# The image runs as nobody by default.  test_cases/ is mounted read-only;
# the verify command writes its working copy to /tmp inside the container
# (an ephemeral tmpfs), so no host files are written and no --user override
# is needed.

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
# Discover test cases
# ---------------------------------------------------------------------------
TEST_CASES_DIR="$PROJECT_ROOT/test_cases"

PASS=0
FAIL=0
declare -a FAILURES=()
# Timing log: lines of "<seconds> <rel_toml>" accumulated for the slowest-10 report.
TIMING_LOG=""

# We iterate over sorted input*.toml files, skipping actual* directories.
while IFS= read -r -d '' input_toml; do
    case_dir="$(dirname "$input_toml")"
    case_basename="$(basename "$case_dir")"

    # Skip directories that are themselves output dirs from a previous run
    if [[ "$case_basename" == actual* ]]; then
        continue
    fi

    # Apply optional filter
    if [[ -n "$FILTER" ]] && ! echo "$input_toml" | grep -qF "$FILTER"; then
        continue
    fi

    rel_toml="${input_toml#"$TEST_CASES_DIR/"}"
    rel_dir="${case_dir#"$TEST_CASES_DIR/"}"
    input_name="$(basename "$input_toml")"

    printf "  %-70s " "$rel_toml"

    t_start=$SECONDS

    # Run fastqrab verify inside the docker container.
    #   - test_cases/ is mounted read-only; verify copies input files into
    #     /tmp (tmpfs) and does all writes there.
    #   - No --output-dir: verify uses a tempdir in /tmp inside the container,
    #     keeping the host filesystem untouched.
    #   - --unsafe-call-prep-sh enables prep.sh / post.sh / test.sh execution.
    if docker run --rm \
            -v "$TEST_CASES_DIR:/test_cases:ro" \
            --tmpfs /tmp:mode=1777 \
            "$IMAGE" \
            verify "/test_cases/$rel_dir/$input_name" \
            --unsafe-call-prep-sh \
            > /tmp/fastqrab_docker_test_stdout.txt 2>/tmp/fastqrab_docker_test_stderr.txt; then
        elapsed=$(( SECONDS - t_start ))
        printf "PASS  %3ds\n" "$elapsed"
        PASS=$((PASS + 1))
    else
        elapsed=$(( SECONDS - t_start ))
        printf "FAIL  %3ds\n" "$elapsed"
        FAIL=$((FAIL + 1))
        FAILURES+=("$rel_toml")
        echo "    stdout: $(cat /tmp/fastqrab_docker_test_stdout.txt)"
        echo "    stderr: $(cat /tmp/fastqrab_docker_test_stderr.txt)"
    fi

    TIMING_LOG="${TIMING_LOG}${elapsed} ${rel_toml}"$'\n'
done < <(find "$TEST_CASES_DIR" -name "input*.toml" -print0 | sort -z)

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
