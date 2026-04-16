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
# The script mounts test_cases/ read-write into the container so the verify
# command can write its temporary working directory there.  Each test case
# gets its own subdirectory under test_cases/<path>/actual_docker/ so that
# docker runs do not collide with normal cargo-test runs that use actual/.

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
SKIP=0
declare -a FAILURES=()

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

    # Use a separate output dir so we don't stomp on local cargo-test runs
    output_dir="$case_dir/actual_docker"

    printf "  %-70s " "$rel_toml"

    # Run fastqrab verify inside the docker container.
    #   - Mount the entire test_cases tree so relative references between
    #     test cases (symlinks, shared sample data) work correctly.
    #   - Pass --output-dir pointing at the per-test actual_docker/ dir on
    #     the mounted volume so the verify command can write there.
    #   - --unsafe-call-prep-sh enables prep.sh / post.sh / test.sh
    #     execution.  These scripts require bash; the docker image ships
    #     busybox which provides /bin/bash (as an ash alias).  Tests that
    #     need tools not present in the minimal image may fail — that is
    #     expected and useful signal.
    if docker run --rm \
            -v "$TEST_CASES_DIR:/test_cases" \
            --tmpfs /tmp \
            --user "$(id -u):$(id -g)" \
            "$IMAGE" \
            verify "/test_cases/$rel_dir/$input_name" \
            --output-dir "/test_cases/$rel_dir/actual_docker" \
            --unsafe-call-prep-sh \
            > /tmp/fastqrab_docker_test_stdout.txt 2>/tmp/fastqrab_docker_test_stderr.txt; then
        echo "PASS"
        PASS=$((PASS + 1))
        # Clean up the actual_docker dir on success to avoid clutter
        rm -rf "$output_dir"
    else
        echo "FAIL"
        FAIL=$((FAIL + 1))
        FAILURES+=("$rel_toml")
        echo "    stdout: $(cat /tmp/fastqrab_docker_test_stdout.txt)"
        echo "    stderr: $(cat /tmp/fastqrab_docker_test_stderr.txt)"
    fi
done < <(find "$TEST_CASES_DIR" -name "input*.toml" -print0 | sort -z)

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "========================================"
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "========================================"

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    echo ""
    echo "Failed test cases:"
    for f in "${FAILURES[@]}"; do
        echo "  - $f"
    done
    exit 1
fi
