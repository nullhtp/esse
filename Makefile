# esse — building it, installing it, and getting it out again.
#
#   make run          run from the source tree
#   make test         everything
#   make app          assemble target/Esse.app
#   make install      put it in /Applications
#   make login-item   start it at login, with no window on screen
#   make uninstall    remove both
#   make icon         redraw resources/esse.icns

APP := Esse.app
BUNDLE := target/$(APP)
PREFIX ?= /Applications
INSTALLED := $(PREFIX)/$(APP)

AGENT := com.nullhtp.esse
# The system-wide summon key, in the spelling the app uses everywhere else.
HOTKEY ?= ctrl-alt-e
AGENTS := $(HOME)/Library/LaunchAgents
AGENT_PLIST := $(AGENTS)/$(AGENT).plist
DOMAIN := gui/$(shell id -u)

.PHONY: run test app install login-item uninstall icon

run:
	cargo run -p esse-app

test:
	cargo test

app:
	@scripts/bundle.sh $(BUNDLE)

install: app
	rm -rf "$(INSTALLED)"
	cp -R "$(BUNDLE)" "$(INSTALLED)"
	@echo "installed: $(INSTALLED)"
	@echo "open it once, then $(HOTKEY) from anywhere"

# The summon key only works while esse is running, so the machine starts it.
login-item: install
	mkdir -p "$(AGENTS)"
	sed -e 's|__PROGRAM__|$(INSTALLED)/Contents/MacOS/esse|' \
		-e 's|__HOTKEY__|$(HOTKEY)|' \
		resources/$(AGENT).plist >"$(AGENT_PLIST)"
	-launchctl bootout $(DOMAIN)/$(AGENT) 2>/dev/null
	launchctl bootstrap $(DOMAIN) "$(AGENT_PLIST)"
	@echo "esse starts at login, hidden, holding $(HOTKEY)"

uninstall:
	-launchctl bootout $(DOMAIN)/$(AGENT) 2>/dev/null
	rm -f "$(AGENT_PLIST)"
	rm -rf "$(INSTALLED)"
	@echo "removed: $(INSTALLED) and the launch agent"
	@echo "your writing is untouched in ~/Documents/Esse"

icon:
	rm -rf target/esse.iconset
	python3 scripts/icon.py target/esse.iconset
	iconutil -c icns target/esse.iconset -o resources/esse.icns
	@echo "redrawn: resources/esse.icns"
