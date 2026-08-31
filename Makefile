# esse — building it, installing it, and getting it out again.
#
#   make run          run from the source tree
#   make test         everything
#   make app          assemble target/Esse.app
#   make install      put it in /Applications
#   make setup        answer the three questions that finish an install
#   make login-item   start it at login, with no window on screen
#   make uninstall    remove both
#   make release      build the zip a release is made of, and the cask
#   make icon         redraw resources/esse.icns

APP := Esse.app
BUNDLE := target/$(APP)
PREFIX ?= /Applications
INSTALLED := $(PREFIX)/$(APP)

# The system-wide summon key, in the spelling the app uses everywhere else.
HOTKEY ?= ctrl-alt-e

.PHONY: run test app install setup login-item uninstall release icon

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
	@echo "now: make setup"

# The three questions that finish an install, asked of the copy in
# /Applications — the same conversation a Homebrew install points at.
setup: install
	@"$(INSTALLED)/Contents/Resources/esse-setup" "$(INSTALLED)/Contents/MacOS/esse"

# The summon key only works while esse is running, so the machine starts it.
# The same script travels inside the bundle, for copies installed by Homebrew.
login-item: install
	@HOTKEY=$(HOTKEY) scripts/login-item.sh "$(INSTALLED)/Contents/MacOS/esse"

uninstall:
	@scripts/login-item.sh --off
	rm -rf "$(INSTALLED)"
	@echo "removed: $(INSTALLED) and the launch agent"
	@echo "your writing is untouched, wherever you keep it"

# The artifact a release is made of, and the cask that points at it.
release:
	@scripts/release.sh

icon:
	rm -rf target/esse.iconset
	python3 scripts/icon.py target/esse.iconset
	iconutil -c icns target/esse.iconset -o resources/esse.icns
	@echo "redrawn: resources/esse.icns"
