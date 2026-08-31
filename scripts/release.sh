#!/usr/bin/env bash
#
# Everything between "the author has a bundle" and "anyone has the app": the
# zip a release is made of, its checksum, the GitHub release, and the cask
# rendered ready to copy into the tap (brew-install design.md).
#
#   scripts/release.sh           build, release, render the cask
#   scripts/release.sh --local   build and render, publish nothing

set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)
tag="v$version"
archive="$root/target/Esse-$version-arm64.zip"
cask="$root/target/esse.rb"
local_only=${1:-}

if [ "$(uname -m)" != "arm64" ]; then
	echo "release: esse ships for Apple silicon only, and this is $(uname -m)" >&2
	exit 1
fi

# A release is a commit, not a working tree: what people download has to be
# something the repository can be checked back against.
if [ -n "$(git -C "$root" status --porcelain)" ]; then
	echo "release: the working tree has uncommitted changes" >&2
	exit 1
fi

"$root/scripts/bundle.sh" "$root/target/Esse.app" >/dev/null

# ditto, not zip: only ditto carries a bundle's symlinks and signature through
# an archive intact (design.md, D2). It is also what Homebrew unpacks with.
rm -f "$archive"
ditto -c -k --keepParent "$root/target/Esse.app" "$archive"

checksum=$(shasum -a 256 "$archive" | cut -d' ' -f1)
sed -e "s/__VERSION__/$version/g" -e "s/__SHA256__/$checksum/g" \
	"$root/resources/cask.rb.in" >"$cask"

echo "archive:  $archive"
echo "sha256:   $checksum"
echo "cask:     $cask"

if [ "$local_only" = "--local" ]; then
	exit 0
fi

if gh release view "$tag" --repo nullhtp/esse >/dev/null 2>&1; then
	gh release upload "$tag" "$archive" --repo nullhtp/esse --clobber
else
	gh release create "$tag" "$archive" --repo nullhtp/esse \
		--title "esse $version" \
		--notes "Install: \`brew install nullhtp/tap/esse\`

A background app for Apple silicon: open it once, then reach it from anywhere
with \`ctrl-alt-e\`. Your writing lives in \`~/Documents/Esse\` as plain files."
fi

echo
echo "now put $cask in nullhtp/homebrew-tap as Casks/esse.rb"
