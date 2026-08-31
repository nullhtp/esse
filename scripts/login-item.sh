#!/usr/bin/env bash
#
# Starts esse at login, hidden, holding its summon key — and stops doing so.
#
# It travels inside the bundle as Contents/Resources/login-item, so a copy
# installed with Homebrew carries its own way to do this and needs no clone of
# the repository (brew-install design.md, D5).
#
#   login-item [executable]     install the launch agent and start it now
#   login-item --off            remove it
#
# HOTKEY names the summon key, in the spelling the app uses everywhere else:
#
#   HOTKEY=cmd-shift-space login-item

set -euo pipefail

label="com.nullhtp.esse"
agents="$HOME/Library/LaunchAgents"
plist="$agents/$label.plist"
domain="gui/$(id -u)"
hotkey="${HOTKEY:-ctrl-alt-e}"

here=$(cd "$(dirname "$0")" && pwd)

if [ "${1:-}" = "--off" ]; then
	launchctl bootout "$domain/$label" 2>/dev/null || true
	rm -f "$plist"
	echo "esse no longer starts at login"
	echo "your writing is untouched in ~/Documents/Esse"
	exit 0
fi

# Either told which binary to run, or found by walking out of the bundle this
# script is sitting in: Contents/Resources/login-item -> Contents/MacOS/esse.
program="${1:-$here/../MacOS/esse}"
if [ ! -x "$program" ]; then
	echo "login-item: no esse executable at $program" >&2
	echo "run it from inside Esse.app, or give it the path" >&2
	exit 1
fi
program=$(cd "$(dirname "$program")" && pwd)/$(basename "$program")

mkdir -p "$agents"
cat >"$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>$label</string>
	<key>ProgramArguments</key>
	<array>
		<string>$program</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<!-- Quitting esse means quitting it: nothing brings it back until the
	     next login. -->
	<key>KeepAlive</key>
	<false/>
	<key>EnvironmentVariables</key>
	<dict>
		<key>ESSE_START_HIDDEN</key>
		<string>1</string>
		<key>ESSE_HOTKEY</key>
		<string>$hotkey</string>
	</dict>
	<key>ProcessType</key>
	<string>Interactive</string>
</dict>
</plist>
PLIST

launchctl bootout "$domain/$label" 2>/dev/null || true
launchctl bootstrap "$domain" "$plist"

echo "esse starts at login, hidden, holding $hotkey"
echo "to stop: $here/$(basename "$0") --off"
