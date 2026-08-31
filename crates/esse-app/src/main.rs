//! esse — the application shell. One window, four screens: Today, the Shelf,
//! and the editor's two modes, Write and Edit.

mod autosave;
mod calendar;
mod data;
mod edit;
mod editor;
mod essay;
mod guidance;
mod help;
mod keymap;
mod line_input;
mod root;
mod shelf;
mod summon;
mod theme;
mod today;
mod write;

use std::time::Instant;

use esse_core::DataDir;
use gpui::{
    actions, prelude::*, px, size, App, Bounds, Focusable, TitlebarOptions, WindowBounds,
    WindowOptions,
};
use gpui_platform::application;

use data::Data;
use root::RootView;

actions!(esse, [Quit]);

fn main() {
    // Before anything else: the launch has a budget, and it is measured from
    // here (today-screen spec, "launch is not the slow part").
    let started = Instant::now();

    // gpui reports platform trouble — a missing font, for one — through `log`
    // and nowhere else; without a logger it fails quietly.
    env_logger::init();

    let dir = match DataDir::open() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("esse: {error}");
            std::process::exit(1);
        }
    };
    let data = Data::open(&dir);

    application().run(move |cx: &mut App| {
        // Out of the Dock and out of the menu bar, before anything is drawn.
        // esse is reached by its key and by its window; this runs here because
        // it is the first moment gpui's own activation policy can be set back
        // (summon.rs, design.md D5).
        summon::become_background_app();

        // Every shortcut the app has vocabulary for comes from one table, which
        // is also what the help overlay reads (keymap.rs, design.md D1). The
        // two baseline sets — the editor's and the capture line's — are the
        // platform's conventions rather than this app's, and stay with the
        // things they belong to.
        cx.bind_keys(keymap::bindings());
        cx.bind_keys(editor::key_bindings());
        cx.bind_keys(line_input::key_bindings());
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        // The window is never let go of: closing it puts esse away, and the
        // summon key brings the same window back with everything in it
        // (root.rs, design.md D4). `cmd-q` is the way out.

        let bounds = Bounds::centered(None, size(px(680.), px(720.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("esse".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| RootView::new(data, window, cx)),
            )
            .unwrap();

        // One key reaches esse from anywhere, and puts it away again
        // (summon-from-anywhere spec).
        summon::listen(window, cx);

        // A login start builds the same window and hides it, so the first
        // summon is as quick as every later one (design.md, D6).
        let hidden = summon::start_hidden();

        // Focus the capture line before the window is on screen: launching the
        // app is already being ready to type.
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx), cx);
                if hidden {
                    cx.hide();
                } else {
                    cx.activate(true);
                }

                // What the launch actually cost, measured to the first frame
                // drawn — the moment the writer could have started typing.
                // Off unless asked for with `ESSE_STARTUP_TIMING=1`, and read
                // through the same logger as everything else, so it costs
                // nothing on an ordinary launch (design.md, D3).
                if std::env::var_os("ESSE_STARTUP_TIMING").is_some() {
                    window.on_next_frame(move |_, _| {
                        log::info!(
                            "startup: {} ms to the first frame",
                            started.elapsed().as_millis()
                        );
                    });
                }
            })
            .unwrap();
    });
}
