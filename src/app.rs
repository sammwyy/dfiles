use crate::{browser::BrowserState, window::WindowRequest};
use creamui_render::{AppBuilder, AppHandle, BlurRegion, WindowHandle, WindowOptions};
use creamui_theme::{Color, Theme};
use dbus::{blocking::Connection, message::MatchRule};
use std::{
    cell::RefCell,
    fs,
    rc::Rc,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

const THEME_PATH: &str = "/org/creamui/Theme";
const THEME_INTERFACE: &str = "org.creamui.Theme";
const THEME_RELOAD: &str = "ReloadTheme";

/// The manager and the portal render the same native browser tree.
pub fn show_window(request: WindowRequest) -> Option<crate::portal::PortalSelection> {
    ensure_wayland_display();
    let browser = BrowserState::new(request);
    let result = browser.result.clone();
    let window = browser.window.clone();
    let options = WindowOptions {
        title: browser.context.request.title.clone(),
        width: 1120,
        height: 760,
        transparent: true,
        decorations: true,
        blur: Some(BlurRegion::Window),
        theme: load_system_theme(),
        ..Default::default()
    };
    let theme_events = Arc::new(Mutex::new(listen_for_theme_reloads()));
    AppBuilder::new()
        .on_started(move |app| {
            let ready_window = window.clone();
            let keys = browser.clone();
            let blur = browser.clone();
            app.append_window(
                options,
                Color::rgba(0, 0, 0, 0),
                move |handle| {
                    crate::components::window_chrome::request_hybrid(&handle);
                    handle.on_keyboard_input(move |event, modifiers| keys.key(event, modifiers));
                    handle.on_focus_lost(move || blur.close_search_if_empty());
                    *ready_window.borrow_mut() = Some(handle);
                },
                move |size| crate::views::build(&browser, size),
            );
            watch_theme_reloads(app, theme_events, window);
        })
        .run();
    let selected = result.lock().expect("browser result").take();
    selected
}

/// Opens the File Explorer preferences instead of a filesystem browser.
pub fn show_settings() {
    ensure_wayland_display();
    let settings = crate::settings::State::load();
    let window = settings.window.clone();
    let options = WindowOptions {
        title: "File Explorer Settings".into(),
        width: 760,
        height: 560,
        transparent: true,
        decorations: true,
        blur: Some(BlurRegion::Window),
        theme: load_system_theme(),
        ..Default::default()
    };
    let theme_events = Arc::new(Mutex::new(listen_for_theme_reloads()));
    AppBuilder::new()
        .on_started(move |app| {
            let ready_window = window.clone();
            let keys = settings.clone();
            app.append_window(
                options,
                Color::rgba(0, 0, 0, 0),
                move |handle| {
                    crate::components::window_chrome::request_hybrid(&handle);
                    handle.on_keyboard_input(move |event, modifiers| keys.key(event, modifiers));
                    *ready_window.borrow_mut() = Some(handle);
                },
                move |size| crate::settings::build(&settings, size),
            );
            watch_theme_reloads(app, theme_events, window);
        })
        .run();
}

fn load_system_theme() -> Theme {
    match creamui_theme_loader::SystemThemeLoader::new().load() {
        Ok(appearance) => {
            if let Some(font_family) = &appearance.font_family {
                creamui_fonts::use_system_font(font_family);
            }
            appearance.theme
        }
        Err(error) => {
            eprintln!("files: failed to load CreamUI system theme: {error}");
            Theme::light()
        }
    }
}

/// Coconut broadcasts this signal after Settings persists a new CreamUI
/// appearance. Keep the receiver on its own D-Bus connection so it cannot
/// block the native UI event loop.
fn listen_for_theme_reloads() -> Receiver<()> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let Ok(connection) = Connection::new_session() else {
            eprintln!("files: session bus unavailable; live theme updates are disabled");
            return;
        };
        if let Err(error) = connection.add_match(
            MatchRule::new_signal(THEME_INTERFACE, THEME_RELOAD).with_path(THEME_PATH),
            move |_: (), _, _| sender.send(()).is_ok(),
        ) {
            eprintln!("files: failed to listen for CreamUI theme updates: {error}");
            return;
        }
        loop {
            if let Err(error) = connection.process(Duration::from_secs(1)) {
                eprintln!("files: CreamUI theme listener stopped: {error}");
                return;
            }
        }
    });
    receiver
}

fn watch_theme_reloads(
    app: AppHandle,
    events: Arc<Mutex<Receiver<()>>>,
    window: Rc<RefCell<Option<WindowHandle>>>,
) {
    let next_app = app.clone();
    let next_events = events.clone();
    let next_window = window.clone();
    app.spawn_background(
        move || events.lock().is_ok_and(|events| events.recv().is_ok()),
        move |changed| {
            if changed {
                if let Some(window) = window.borrow().as_ref() {
                    window.set_theme(load_system_theme());
                }
                watch_theme_reloads(next_app, next_events, next_window);
            }
        },
    );
}
fn ensure_wayland_display() {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return;
    }
    let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return;
    };
    use std::os::unix::fs::FileTypeExt;
    let mut sockets = fs::read_dir(runtime)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_socket()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("wayland-"))
        .collect::<Vec<_>>();
    sockets.sort();
    if let Some(socket) = sockets.first() {
        std::env::set_var("WAYLAND_DISPLAY", socket);
    }
}
