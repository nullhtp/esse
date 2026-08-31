#!/usr/bin/env bash
#
# Assembles Esse.app around a release build. The whole job is a directory, a
# binary, a plist and an icon — which is why there is no packaging crate here
# (install-and-summon design.md, D8).

set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
bundle="${1:-$root/target/Esse.app}"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)

cargo build --release --manifest-path "$root/Cargo.toml" -p esse-app

rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"

cp "$root/target/release/esse" "$bundle/Contents/MacOS/esse"
cp "$root/resources/esse.icns" "$bundle/Contents/Resources/esse.icns"
sed "s/__VERSION__/$version/g" "$root/resources/Info.plist" >"$bundle/Contents/Info.plist"
printf 'APPL????' >"$bundle/Contents/PkgInfo"

# Ad-hoc signature: unsigned binaries are refused outright on Apple silicon,
# and the app is built on the machine it runs on, so there is nothing to
# notarise.
codesign --force --sign - "$bundle" >/dev/null 2>&1 || true

# A new bundle at an old path keeps the old icon until Finder is told.
touch "$bundle"

echo "$bundle"
