#!/usr/bin/env bash

set -euo pipefail

EXPECTED_HEADER_A='<<<<<<< Conflict 1 of 1
%%%%%%% Changes from base to side #1'

EXPECTED_FOOTER_A='+++++++ Contents of side #2
>>>>>>> Conflict 1 of 1 ends'

EXPECTED_HEADER_B='<<<<<<< Conflict 1 of 1
+++++++ Contents of side #1
%%%%%%% Changes from base to side #2'

EXPECTED_FOOTER_B='>>>>>>> Conflict 1 of 1 ends'

die() {
  echo "$@" >&2
  exit 1
}

[[ $# -eq 2 ]] || die "usage: $0 <conflicting> <renamed>"

conflicting="$1"
renamed="$2"
lines="$(wc -l < "$conflicting")"

if [[ "$(head -2 "$conflicting")" == "$EXPECTED_HEADER_A" \
      && "$(tail -2 "$conflicting")" == "$EXPECTED_FOOTER_A" ]]; then
  conflicting_diff="$(sed "1,2d;$((lines-1)),${lines}d" "$conflicting")"
elif [[ "$(head -3 "$conflicting")" == "$EXPECTED_HEADER_B" \
        && "$(tail -1 "$conflicting")" == "$EXPECTED_FOOTER_B" ]]; then
  conflicting_diff="$(sed "1,3d;${lines}d" "$conflicting")"
else
  die "file does not look like a conflicting deletion: $1"
fi


orig=$(mktemp)
edited=$(mktemp)
trap 'rm -- "$orig" "$edited"' EXIT

echo "$conflicting_diff" | sed '/^+/d;s/^.//' > "$orig"
echo "$conflicting_diff" | sed '/^-/d;s/^.//' > "$edited"

out="$(set +e; diff3 -m "$edited" "$orig" "$renamed")" || rc=$?
[[ "${rc:=0}" -ne 2 ]] || exit 2

if command -v delta >/dev/null; then
  delta --paging=never "$renamed" - <<<"$out" ||:
else
  diff -u "$renamed" - <<<"$out" ||:
fi

[[ "$rc" -eq 1 ]] && echo 'WARNING: has conflicts!'

while :; do
  read -rp 'ok? [y/n] ' answer
  case "$answer" in
    [Yy]) break ;;
    [Nn]) exit ;;
  esac
done

cp "$renamed"{,.bak}
echo "$out" > "$renamed"
rm "$renamed.bak" "$conflicting"
