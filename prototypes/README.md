# prototypes

Throwaway prototypes for choosing the GUI framework
(`openspec/changes/editor-framework-prototype`). The code is deliberately
rough — it lives until the decision is written into the Decision Record, and is
frozen afterwards.

A separate Cargo workspace with its own lock file: it is not part of the main
crate.

```
gpui-editor/     the editor prototype on gpui
```

The `markdown-lite` parser survived the spike and moved into production — it
now lives in `crates/markdown-lite` of the main workspace, and the prototype
depends on it by path.

## Running

```
cd prototypes
cargo run -p gpui-editor
cargo test              # 63 tests, no GUI needed
```

The first build is long: it pulls gpui out of the Zed repository with the whole
platform layer.

Requirements besides Rust:

- toolchain 1.97.1 — installed automatically from `rust-toolchain.toml`;
- on macOS, the Metal component:
  `xcodebuild -downloadComponent MetalToolchain` (~700 MB), without which the
  build dies compiling gpui's shaders.

A gpui trap: `gpui_platform` has to be pulled in with `features = ["font-kit"]`
— without it the window opens but no text is drawn at all. The only sign is a
warning through `log`, which is why the prototype calls `env_logger::init()`;
running it with `RUST_LOG=warn` is worth it.

## Keys

| | |
|---|---|
| arrows, `home`/`end`, `cmd-←`/`cmd-→` | the cursor |
| `shift` + arrows, `shift-home`/`end` | selection |
| mouse: click, drag | cursor and selection |
| `cmd-c` / `cmd-x` / `cmd-v` | the clipboard |
| `cmd-z` / `cmd-shift-z` | undo and redo |
| `cmd-a` | select all |
| the mouse wheel | scrolling (works with typewriter mode on, too) |
| `cmd-shift-t` | typewriter mode (on by default) |
| `cmd-shift-l` | load a 20,000-character essay |
| `cmd-q` | quit |

## What has to be checked by hand

What tests cannot check — tasks 2.3, 3.8 and group 4 in `tasks.md`:

1. **Cyrillic and IME** (a hard gate). Type a paragraph in Russian. Switch to a
   layout that goes through an IME and type a word with composition — the
   characters being composed must be underlined and inserted without losses or
   reordering.
2. **Markup in place.** Type `**bold**`, `*italic*`, `# heading`. Move the
   cursor off the line: the markup characters disappear, the style stays. Come
   back to the line: the characters are visible again.
3. **Other markdown.** `- list`, `[link](url)`, `` `code` `` must stay text.
4. **Typewriter mode.** The cursor's line stays centred while typing and on
   `enter`.
5. **Responsiveness.** `cmd-shift-l`, then type and scroll: there must be no
   delay between a keypress and the character on screen. The computed part of
   the frame is measured by `cargo test --test performance -- --nocapture`; the
   eye is left to judge shaping and drawing.
6. **The subjective gate.** Twenty minutes of real writing: do you want to
   write here.

The results are in the Decision Record in `design.md`.
