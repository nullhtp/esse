//! esse — the application shell. One window, three screens: Today, and the
//! editor's two modes, Write and Edit.

mod autosave;
mod data;
mod edit;
mod editor;
mod root;
mod spark_input;
mod theme;
mod today;
mod write;

use esse_core::DataDir;
use gpui::{
    actions, prelude::*, px, size, App, Bounds, Focusable, KeyBinding, TitlebarOptions,
    WindowBounds, WindowOptions,
};
use gpui_platform::application;

use data::Data;
use root::RootView;
use spark_input::{Backspace, Delete, End, Home, Left, Right, Submit};

actions!(esse, [Quit]);

fn main() {
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
            KeyBinding::new("enter", Submit, Some("SparkInput")),
            KeyBinding::new("backspace", Backspace, Some("SparkInput")),
            KeyBinding::new("delete", Delete, Some("SparkInput")),
            KeyBinding::new("left", Left, Some("SparkInput")),
            KeyBinding::new("right", Right, Some("SparkInput")),
            KeyBinding::new("home", Home, Some("SparkInput")),
            KeyBinding::new("end", End, Some("SparkInput")),
            KeyBinding::new("cmd-left", Home, Some("SparkInput")),
            KeyBinding::new("cmd-right", End, Some("SparkInput")),
            // Leaving either editor mode is one explicit key, and crossing
            // between them is one more — the same gesture in both directions,
            // so it stays in the hand rather than in the head (design.md, D3).
            KeyBinding::new("escape", write::Leave, Some("Write")),
            KeyBinding::new("cmd-e", write::Switch, Some("Write")),
            KeyBinding::new("escape", edit::Leave, Some("Edit")),
            KeyBinding::new("cmd-e", edit::Switch, Some("Edit")),
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
            })
            .unwrap();
    });
}
