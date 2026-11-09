//! Holm: a lean Dynamic Island for Windows, with macOS and Linux in development.
//!
//! Rust owns everything that touches the operating system (window placement and click-through,
//! media sessions, system readings, agent hooks, clipboard, radios, volume). The Svelte frontend
//! in ../src draws the island and the settings window and talks to these modules through the
//! commands registered below and the events each module emits (see docs/ARCHITECTURE.md).
//!
//! Modules that talk to the system have a Windows file here and a macOS/Linux file in
//! `portable/` with the same commands and events; `platform_info` tells the UI which features a
//! system has, so the rest hides instead of breaking.

mod agents;
#[cfg_attr(not(windows), path = "portable/appvolume.rs")]
mod appvolume;
#[cfg_attr(not(windows), path = "portable/clipboard.rs")]
mod clipboard;
mod config;
#[cfg(windows)]
mod games;
mod harness;
#[cfg_attr(not(windows), path = "portable/island.rs")]
mod island;
#[cfg_attr(not(windows), path = "portable/media.rs")]
mod media;
mod net;
#[cfg_attr(not(windows), path = "portable/quiet.rs")]
mod quiet;
mod quotas;
#[cfg_attr(not(windows), path = "portable/radios.rs")]
mod radios;
#[cfg_attr(not(windows), path = "portable/shell.rs")]
mod shell;
#[cfg_attr(not(windows), path = "portable/system.rs")]
mod system;
mod util;
#[cfg_attr(not(windows), path = "portable/volume.rs")]
mod volume;

use std::sync::Mutex;

use serde_json::Value;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry,
};

use config::Config;

/// Which system Holm is running on and which of its features work there. The UI hides what a
/// system doesn't have (on macOS and Linux, while their versions are being built).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlatformInfo {
    os: &'static str,
    media: bool,
    app_volume: bool,
    volume: bool,
    radios: bool,
    clipboard: bool,
    games: bool,
    quiet: bool,
    /// Knows which maximized app is in front (Make way, Learn).
    front_app: bool,
    battery: bool,
    system_stats: bool,
}

#[tauri::command]
fn platform_info() -> PlatformInfo {
    let windows = cfg!(windows);
    let linux = cfg!(target_os = "linux");
    PlatformInfo {
        os: if windows {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        },
        media: windows,
        app_volume: windows,
        volume: windows,
        radios: windows,
        clipboard: windows,
        games: windows,
        quiet: windows,
        front_app: windows,
        battery: windows || linux,
        system_stats: windows || linux,
    }
}

struct TrayItems {
    hide: MenuItem<Wry>,
}

pub fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("index.html#settings".into()),
    )
    .title("Holm · Settings")
    .inner_size(940.0, 680.0)
    .min_inner_size(800.0, 560.0)
    .center()
    .resizable(true)
    .decorations(true)
    .visible(true)
    .build()?;
    Ok(())
}

/// Reacts to the settings that Rust applies itself; the frontend applies the rest.
pub fn on_settings_changed(app: &AppHandle, patch: &Value) {
    if patch.get("monitor").is_some() {
        island::reposition(app);
    }
    if let Some(on) = patch.get("autostart").and_then(Value::as_bool) {
        if let Err(err) = shell::set_autostart(on) {
            util::log(app.state::<Config>().dir(), &format!("autostart: {err}"));
        }
    }
}

fn toggle_hidden(app: &AppHandle) {
    set_hidden(app, !island::user_hidden(app));
}

/// Hides or shows the island on request (tray, hide shortcut) and keeps the tray label in step.
pub fn set_hidden(app: &AppHandle, hidden: bool) {
    island::set_user_hidden(app, hidden);
    if let Some(items) = app.try_state::<Mutex<TrayItems>>() {
        if let Ok(items) = items.lock() {
            let _ = items
                .hide
                .set_text(if hidden { "Show island" } else { "Hide island" });
        }
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open island", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide island", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Holm", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &settings, &hide, &separator, &quit])?;
    app.manage(Mutex::new(TrayItems { hide: hide.clone() }));

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Holm")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if island::user_hidden(app) {
                    toggle_hidden(app);
                }
                let _ = app.emit("island://toggle", ());
            }
            "settings" => {
                let _ = open_settings_window(app);
            }
            "hide" => toggle_hidden(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = open_settings_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// WebView2 reads its browser flags once, when it starts, so the graphics setting is applied here,
/// before any window exists. Software compositing measured ~120 MB private memory against ~185 MB
/// with the GPU process, at a small CPU cost while the island morphs. A flag set by the user in
/// the environment wins.
fn apply_graphics_setting() {
    if !cfg!(windows) {
        return;
    }
    const VAR: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";
    if std::env::var_os(VAR).is_some() {
        return;
    }
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return;
    };
    // Same folder as Config (app_config_dir = %APPDATA%\<identifier>).
    let path = std::path::PathBuf::from(appdata)
        .join(IDENTIFIER)
        .join("settings.json");
    // Measured on the release build (i7-11700): Software and these flags together hold about
    // 120 MB private instead of about 260 MB, at a few percent of one core more while the island
    // is open. The flags keep Tauri's own defaults (this variable replaces them), skip the spare
    // page process and the background downloads a local app never needs, and put both windows in
    // one page process.
    const LEAN: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,\
        SpareRendererForSitePerProcess,msEdgeTranslate \
        --disable-background-networking --disable-component-update --renderer-process-limit=1";
    let hardware = util::read_json(&path)
        .get("graphics")
        .and_then(Value::as_str)
        == Some("gpu");
    let args = if hardware {
        LEAN.to_string()
    } else {
        format!("--disable-gpu {LEAN}")
    };
    std::env::set_var(VAR, args);
}

/// tauri.conf.json's identifier, which names the settings folder.
const IDENTIFIER: &str = "in.tenseitech.holm";
/// The identifier before the rename to Holm.
const LEGACY_IDENTIFIER: &str = "in.tenseitech.island";

/// Settings and data from before the rename are copied into the new folder once, the first time
/// Holm starts with no settings of its own. The old folder is left as it was.
fn migrate_legacy_settings() {
    if !cfg!(windows) {
        return;
    }
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return;
    };
    let root = std::path::PathBuf::from(appdata);
    let (old, new) = (root.join(LEGACY_IDENTIFIER), root.join(IDENTIFIER));
    if new.join("settings.json").exists() || !old.join("settings.json").exists() {
        return;
    }
    let _ = std::fs::create_dir_all(&new);
    for name in ["settings.json", "data.json"] {
        let _ = std::fs::copy(old.join(name), new.join(name));
    }
}

pub fn run() {
    migrate_legacy_settings();
    shell::migrate_legacy_autostart();
    shell::repair_autostart();
    apply_graphics_setting();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Starting the app again opens its settings instead of a second island.
            let _ = open_settings_window(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // A menu bar app on macOS: no Dock icon, no app menu.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle().clone();
            app.manage(Config::load(&handle));
            island::setup(&handle)?;
            setup_tray(&handle)?;
            media::start(handle.clone());
            system::start(handle.clone());
            agents::start(handle.clone());
            quotas::start(handle.clone());
            quiet::start(handle.clone());
            clipboard::start(handle.clone());
            volume::start(handle.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::settings_get,
            config::settings_patch,
            config::data_get,
            config::data_set,
            island::island_set_rect,
            island::island_release_focus,
            island::island_set_hidden,
            island::island_summon,
            island::island_state,
            island::island_cursor,
            island::monitors_list,
            island::open_settings,
            media::media_control,
            media::media_refresh,
            system::system_watch,
            system::power_get,
            agents::agents_state,
            agents::agents_decide,
            agents::agents_clear,
            agents::hooks_status,
            agents::hooks_install,
            agents::hooks_uninstall,
            quotas::quotas_refresh,
            quotas::quotas_get,
            harness::harness_scan,
            appvolume::app_volume_get,
            appvolume::app_volume_set,
            appvolume::app_volume_mute,
            quiet::quiet_get,
            clipboard::clipboard_list,
            clipboard::clipboard_copy,
            clipboard::clipboard_clear,
            radios::radios_get,
            radios::radio_set,
            volume::volume_get,
            volume::volume_set,
            volume::volume_mute,
            shell::autostart_get,
            shell::autostart_set,
            shell::open_path,
            shell::reveal_path,
            shell::open_url,
            shell::path_exists,
            net::fetch_json,
            platform_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Holm");
}
