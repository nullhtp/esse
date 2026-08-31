#!/usr/bin/env bash
#
# The one command that finishes an install: where the essays live, whether esse
# waits at login, and which key brings it forward.
#
# A Homebrew cask cannot ask questions — `brew install` is not a conversation —
# so the questions are asked here, once, in the terminal the writer is already
# in. It travels inside the bundle as Contents/Resources/esse-setup, so an
# installed copy carries its own setup and needs no clone of the repository
# (guided-install design.md, D1).
#
#   esse-setup [executable]
#
# It asks; scripts/login-item.sh acts (D2). Both answers are recorded in one
# small file that the app only ever reads: esse has no settings screen, and
# this is not the beginning of one (D3).

set -euo pipefail

conf="$HOME/Library/Preferences/com.nullhtp.esse.conf"
default_dir="$HOME/Documents/Esse"
default_hotkey="ctrl-alt-e"
# Where an installation older than the visible folder still keeps its files.
# The app moves it across by itself on the first launch; setup only looks here
# so that a writer who names a folder before ever opening esse takes their
# essays with them (D6).
hidden_dir="$HOME/Library/Application Support/esse"

here=$(cd "$(dirname "$0")" && pwd)

# Inside the bundle these three sit together: Contents/Resources/esse-setup,
# Contents/Resources/login-item, and Contents/MacOS/esse one directory over.
# From the source tree the Makefile names the executable instead.
esse="${1:-$here/../MacOS/esse}"
if [ ! -x "$esse" ]; then
	echo "esse-setup: no esse executable at $esse" >&2
	echo "run it from inside Esse.app, or give it the path" >&2
	exit 1
fi
esse=$(cd "$(dirname "$esse")" && pwd)/$(basename "$esse")

login_item="$here/login-item"
if [ ! -x "$login_item" ]; then
	login_item="$here/login-item.sh"
fi
if [ ! -x "$login_item" ]; then
	echo "esse-setup: cannot find the login-item script next to $0" >&2
	exit 1
fi

# ---------------------------------------------------------------- the answers

# One recorded value, or nothing at all. Comments, blank lines and keys this
# version has no use for are skipped, exactly as the app skips them.
read_conf() {
	if [ ! -f "$conf" ]; then
		return 0
	fi
	awk -v want="$1" '
		/^[[:space:]]*#/ { next }
		index($0, "=") == 0 { next }
		{
			key = substr($0, 1, index($0, "=") - 1)
			value = substr($0, index($0, "=") + 1)
			gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
			gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
			if (key == want && length(value)) found = value
		}
		END { if (length(found)) print found }
	' "$conf"
}

# `~/Writing/Essays` as a person types it, into a path the machine can open.
# The tildes here are meant to stay tildes: one is a pattern, one is text.
# shellcheck disable=SC2088
expand() {
	case "$1" in
	"~") printf '%s' "$HOME" ;;
	"~/"*) printf '%s/%s' "$HOME" "${1#\~/}" ;;
	*) printf '%s' "$1" ;;
	esac
}

# And back again, because `~/Documents/Esse` is how a person reads a path.
# shellcheck disable=SC2088
pretty() {
	case "$1" in
	"$HOME"/*) printf '~/%s' "${1#"$HOME"/}" ;;
	"$HOME") printf '~' ;;
	*) printf '%s' "$1" ;;
	esac
}

# Whether a folder is esse's: any one of the three things it writes.
holds_writing() {
	if [ ! -d "$1" ]; then
		return 1
	fi
	[ -f "$1/sparks.jsonl" ] || [ -f "$1/sessions.jsonl" ] || [ -d "$1/essays" ]
}

count_essays() {
	if [ -d "$1/essays" ]; then
		find "$1/essays" -maxdepth 1 -type f -name '*.md' | wc -l | tr -d ' '
	else
		echo 0
	fi
}

count_sparks() {
	if [ -f "$1/sparks.jsonl" ]; then
		awk 'NF' "$1/sparks.jsonl" | wc -l | tr -d ' '
	else
		echo 0
	fi
}

# "1 essay", "12 essays" — the count is about someone's writing, so it is worth
# getting the grammar right.
plural() {
	if [ "$1" = "1" ]; then
		printf '1 %s' "$2"
	else
		printf '%s %ss' "$1" "$2"
	fi
}

recorded_dir=$(read_conf data_dir)
recorded_hotkey=$(read_conf hotkey)

if [ -n "$recorded_dir" ]; then
	folder=$(expand "$recorded_dir")
else
	folder="$default_dir"
fi
hotkey="${recorded_hotkey:-$default_hotkey}"

if "$login_item" --status >/dev/null 2>&1; then
	at_login_now=y
else
	at_login_now=n
fi

# What the login question offers: what is already true, or an earlier "no" that
# was meant, and otherwise yes — a key that only answers while the app happens
# to be running is not a key you learn to reach for (D7).
if [ "$at_login_now" = y ]; then
	at_login=y
elif [ "$(read_conf at_login)" = no ]; then
	at_login=n
else
	at_login=y
fi

# Where the writing is now, which is not always the folder the app would open:
# an installation older than the visible folder still has it out of sight.
source_dir="$folder"
if ! holds_writing "$source_dir" && [ -z "$recorded_dir" ] && holds_writing "$hidden_dir"; then
	source_dir="$hidden_dir"
fi

# ------------------------------------------------------------ saying it plain

state() {
	printf '    %-22s %s\n' "$hotkey" 'brings esse forward, and puts it away'
	printf '    %-22s %s\n' "$(pretty "$folder")" 'is where your essays live'
	if [ "$at_login" = y ]; then
		printf '    %-22s %s\n' 'at login' 'esse is already waiting'
	else
		printf '    %-22s %s\n' 'not at login' 'esse waits only once you open it'
	fi
}

# Nothing to answer the questions with: say what is true and change nothing.
# Taking the defaults quietly here would let a script install a launch agent
# nobody asked for (D8).
if [ ! -t 0 ]; then
	# What is true, not what would be offered: nothing here is a question.
	at_login="$at_login_now"
	printf '\nesse is set up like this:\n\n'
	state
	printf '\n  To change any of it, run this from a terminal:\n    %s\n\n' \
		"$here/$(basename "$0")"
	exit 0
fi

# ------------------------------------------------------------- the three asks

answer=""

# The prompt, and the line typed back. End of input anywhere in the
# conversation means the same as never having started it: nothing has been
# acted on yet, so nothing is undone.
ask() {
	printf '  %s › ' "$1"
	if ! IFS= read -r answer; then
		printf '\n\n  Nothing was changed.\n\n'
		exit 0
	fi
}

confirm() {
	local hint
	if [ "$2" = y ]; then hint='[Y/n]'; else hint='[y/N]'; fi
	while true; do
		ask "$1 $hint"
		case "${answer:-$2}" in
		[Yy] | [Yy][Ee][Ss]) return 0 ;;
		[Nn] | [Nn][Oo]) return 1 ;;
		*) printf '  yes or no.\n' ;;
		esac
	done
}

printf '\nesse — three questions, and then you can forget this file exists.\n\n'

# 1. Where the essays live.
printf '  Your essays are plain files in a folder you can open, read and\n'
printf '  back up like any other. Where should that folder be?\n'
move_from=""
left_behind=""
while true; do
	move_from=""
	left_behind=""
	ask "[$(pretty "$folder")]"
	candidate=$(expand "${answer:-$(pretty "$folder")}")
	candidate=${candidate%/}

	if [ "$candidate" = "$source_dir" ]; then
		folder="$candidate"
		break
	fi

	# Two folders of the same files are two truths, and choosing between them
	# is not setup's to do (D6).
	if holds_writing "$candidate" && holds_writing "$source_dir"; then
		printf '\n  Both of these already hold essays:\n    %s\n    %s\n' \
			"$(pretty "$source_dir")" "$(pretty "$candidate")"
		printf '  esse reads one folder, and merging two is not mine to do.\n'
		printf '  Name one of them, or a folder that is empty.\n\n'
		continue
	fi

	if ! mkdir -p "$candidate" 2>/dev/null; then
		printf '\n  I could not make a folder at %s.\n' "$(pretty "$candidate")"
		printf '  Try another path.\n\n'
		continue
	fi

	folder="$candidate"

	# One case has nothing to ask about: an installation older than the visible
	# folder, keeping the default answer. The app carries that one across by
	# itself, once, on the next launch (local-storage spec).
	offer=y
	if [ "$source_dir" = "$hidden_dir" ] && [ "$folder" = "$default_dir" ]; then
		offer=n
	fi

	if [ "$offer" = y ] && holds_writing "$source_dir"; then
		printf '\n  %s holds %s and %s.\n' \
			"$(pretty "$source_dir")" \
			"$(plural "$(count_essays "$source_dir")" essay)" \
			"$(plural "$(count_sparks "$source_dir")" spark)"
		if confirm "Move all of it to $(pretty "$folder")?" y; then
			move_from="$source_dir"
		else
			left_behind="$source_dir"
		fi
	fi
	break
done

# 2. Waiting at login.
printf '\n  esse has no Dock icon and no menu bar. It waits in the background\n'
printf '  and comes when you call it — which it can only do while it runs.\n'
if confirm 'Start esse at login, holding its key?' "$at_login"; then
	at_login=y
else
	at_login=n
fi

# 3. The key.
printf '\n  Reach esse from anywhere with   %s\n' "$hotkey"
printf '  Press it in any app and esse is in front, ready to type; press it\n'
printf '  again and it is gone, with your words already on disk.\n'
while true; do
	ask 'Another combination? [enter to keep]'
	if [ -z "$answer" ]; then
		break
	fi
	if "$esse" --check-hotkey "$answer" 2>/dev/null; then
		hotkey="$answer"
		break
	fi
	printf '\n  esse cannot read "%s".\n' "$answer"
	printf '  Spell it the way the app does: cmd-shift-space, ctrl-alt-e, alt-f1.\n\n'
done

# ------------------------------------------------------------------- doing it

# The writing first: a location is recorded only once the essays are actually
# there (D6).
if [ -n "$move_from" ]; then
	if [ ! -e "$folder" ] || rmdir "$folder" 2>/dev/null; then
		# One rename, which either happens or does not.
		mkdir -p "$(dirname "$folder")"
		mv "$move_from" "$folder"
	else
		# The folder was already there with something else in it, so only the
		# three things esse owns move.
		for item in essays sparks.jsonl sessions.jsonl; do
			if [ -e "$move_from/$item" ]; then
				mv "$move_from/$item" "$folder/"
			fi
		done
		rmdir "$move_from" 2>/dev/null || true
	fi
	printf '\n  Moved your writing to %s\n' "$(pretty "$folder")"
elif [ -n "$left_behind" ]; then
	printf '\n  Your earlier writing stayed in %s\n' "$(pretty "$left_behind")"
fi

# Only what departs from the defaults is written down, so a machine that took
# every default is indistinguishable from one that never ran setup (D4).
lines=""
if [ "$folder" != "$default_dir" ]; then
	# Written the way it was typed, `~` and all: the app expands it, and a
	# person reading the file sees a path they recognise.
	lines="${lines}data_dir=$(pretty "$folder")
"
fi
if [ "$hotkey" != "$default_hotkey" ]; then
	lines="${lines}hotkey=$hotkey
"
fi
if [ "$at_login" = n ]; then
	# Setup's own memory, which the app never reads: without it, pressing
	# enter through a later run would undo a deliberate no (D7).
	lines="${lines}at_login=no
"
fi
if [ -z "$lines" ]; then
	rm -f "$conf"
else
	mkdir -p "$(dirname "$conf")"
	{
		echo "# esse — written by esse-setup, read by esse, and by nothing else."
		echo "# Only what differs from the defaults is here."
		printf '%s' "$lines"
	} >"$conf"
fi

# The launch agent is rewritten in the same act, so the file and the agent
# cannot come to disagree about which key esse holds (D5).
if [ "$at_login" = y ]; then
	HOTKEY="$hotkey" "$login_item" "$esse" >/dev/null
else
	"$login_item" --off >/dev/null
fi

# ------------------------------------------------------------------- and done

printf '\n  Done.\n\n'
state

if pgrep -x esse >/dev/null 2>&1; then
	printf '\n  esse is running with the answers it started with. Quit it with\n'
	printf '  cmd-q, then press %s — that copy is the one that heard all this.\n' "$hotkey"
elif [ "$at_login" = y ]; then
	printf '\n  Press %s and write the first line.\n' "$hotkey"
else
	printf '\n  Open Esse.app once, then %s reaches it from anywhere.\n' "$hotkey"
fi
printf '\n'
