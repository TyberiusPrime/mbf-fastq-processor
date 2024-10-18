#!/usr/bin/env bash
BINARY_REPORTED_VERSION=v`cat Cargo.toml |grep ^version | head -n1 | cut -f 2 -d "\""`
cat <<EOF >/tmp/release.json
{
  "release": {
	"tag_name": "${BINARY_REPORTED_VERSION}"
  }
}
EOF

nix shell nixpkgs#act --command act release -j build -e /tmp/release.json
