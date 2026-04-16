#!/usr/bin/env bash
#set -euo pipefail # we need to check it


# Remove all paths containing 'rapidgzip' from PATH
PATH=$(echo "$PATH" | tr ':' '\n' | grep -v 'rapidgzip' | tr '\n' ':' | sed 's/:$//')
export PATH

# When rapidgzip lives directly in a standard bin dir (e.g. /bin/rapidgzip in
# a docker container) the name-based filter above won't remove it.  Build a
# shadow directory that contains everything on the current PATH except the
# rapidgzip binary and use that instead.
if command -v rapidgzip >/dev/null 2>&1; then
    _shadow=$(mktemp -d)
    while IFS= read -r _dir; do
        [ -d "$_dir" ] || continue
        for _f in "$_dir"/*; do
            [ -x "$_f" ] || continue
            _name=$(basename "$_f")
            [ "$_name" = "rapidgzip" ] && continue
            [ -e "$_shadow/$_name" ] || ln -sf "$_f" "$_shadow/$_name"
        done
    done <<< "$(echo "$PATH" | tr ':' '\n')"
    export PATH="$_shadow"
    echo "shadow path (no rapidgzip): $PATH"
fi



# make sure it's not return code 0
if $PROCESSOR_CMD process "$CONFIG_FILE" >stdout 2>stderr
  then
    echo "Expected non-zero exit code, but got zero"
    echo "ran $PROCESSOR_CMD process $CONFIG_FILE"
    echo "stdout from run was"
    cat stdout
    echo "stderr from run was"
    cat stderr

    cat "$CONFIG_FILE"
    ls
    cat output_read1.fq
    exit 1
fi

# make sure expected_panic.txt contents are in stderr
EXPECTED_STRING="Make sure you have a rapidgzip binary on your path."

stderr=$(cat stderr)
if ! grep -q "$EXPECTED_STRING" stderr; then
    echo "Expected panic message not found in stderr"
    echo "stderr: $stderr"
    exit 1
fi
