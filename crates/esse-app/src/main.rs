//! esse — the application shell. One window, four screens: Today, the Shelf,
//! and the editor's two modes, Write and Edit.

mod autosave;
mod calendar;
mod data;
mod edit;
mod editor;
mod essay;
mod line_input;
mod root;
mod shelf;
mod theme;
mod today;
mod write;

use std::time::Instant;

use esse_core::DataDir;
use gpui::{
    actions, prelude::*, px, size, App, Bounds, Focusable, KeyBinding, TitlebarOptions,
    WindowBounds, WindowOptions,
};
use gpui_platform::application;

use data::Data;
use line_input::{Backspace, Delete, End, Home, Left, Paste, Right, Submit};
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
        cx.bind_keys([
            KeyBinding::new("enter", Submit, Some("LineInput")),
            KeyBinding::new("backspace", Backspace, Some("LineInput")),
            KeyBinding::new("delete", Delete, Some("LineInput")),
            KeyBinding::new("left", Left, Some("LineInput")),
            KeyBinding::new("right", Right, Some("LineInput")),
            KeyBinding::new("home", Home, Some("LineInput")),
            KeyBinding::new("end", End, Some("LineInput")),
            KeyBinding::new("cmd-left", Home, Some("LineInput")),
            KeyBinding::new("cmd-right", End, Some("LineInput")),
            KeyBinding::new("cmd-v", Paste, Some("LineInput")),
            // Leaving either editor mode is one explicit key, and crossing
            // between them is one more — the same gesture in both directions,
            // so it stays in the hand rather than in the head (design.md, D3).
            KeyBinding::new("escape", write::Leave, Some("Write")),
            KeyBinding::new("cmd-e", write::Switch, Some("Write")),
            KeyBinding::new("escape", edit::Leave, Some("Edit")),
            KeyBinding::new("cmd-e", edit::Switch, Some("Edit")),
            // Today's two moves, so the daily loop needs no pointer: the
            // Write button, and the way to the Shelf. Both take a modifier —
            // plain keys belong to the capture line (keyboard-shortcuts spec).
            KeyBinding::new("cmd-enter", today::Write, Some("Today")),
            KeyBinding::new("cmd-l", today::Shelf, Some("Today")),
            // The Shelf is left the way the editor rooms are left — and also
            // by the gesture that opened it, so the trip there and back is
            // one key pressed twice.
            KeyBinding::new("escape", shelf::Leave, Some("Shelf")),
            KeyBinding::new("cmd-l", shelf::Leave, Some("Shelf")),
            KeyBinding::new("cmd-q", Quit, None),
        ]);
        cx.bind_keys(editor::key_bindings());
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        // Closing the only window ends the app; esse has nothing to stay
        // resident for.
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

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

        // Focus the capture line before the window is on screen: launching the
        // app is already being ready to type.
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx), cx);
                cx.activate(true);

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
