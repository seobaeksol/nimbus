mod explorer;
mod filesystem;
mod git;
mod model;
mod operation;
mod persistence;
mod preview;
mod windows_integration;

use explorer::Explorer;
use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};
use gpui_component::{Root, Theme, ThemeMode, TitleBar};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nimbus=info".into()),
        )
        .init();

    let requested_workspace = requested_workspace_argument();
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_bounds: Some(WindowBounds::centered(size(px(1100.), px(720.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                window.activate_window();
                window.set_window_title("Nimbus");
                Theme::change(ThemeMode::Dark, Some(window), cx);

                let explorer = cx.new(|cx| Explorer::new(requested_workspace.clone(), window, cx));
                cx.new(|cx| Root::new(explorer, window, cx))
            })
            .expect("failed to open the Nimbus window");
        })
        .detach();
    });
}

fn requested_workspace_argument() -> Option<String> {
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--workspace" || argument == "-w" {
            return arguments.next();
        }
        if let Some(name) = argument.strip_prefix("--workspace=") {
            return Some(name.to_string());
        }
    }
    None
}
