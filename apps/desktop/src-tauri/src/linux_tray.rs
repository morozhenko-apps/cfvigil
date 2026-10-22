//! Native StatusNotifierItem for KDE's left-click Activate semantics.
use crate::{hide_card, show_card, show_window};
use ksni::{blocking::TrayMethods as _, menu::StandardItem, Icon, MenuItem, Tray};
use tauri::{App, AppHandle, Manager};

struct CFVigilTray {
    app: AppHandle,
}

fn tray_argb() -> (Vec<u8>, u32, u32) {
    let icon = image::load_from_memory(include_bytes!("../icons/icon.png"))
        .expect("CFVigil icon PNG is invalid")
        .resize_exact(32, 32, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (width, height) = icon.dimensions();
    let mut argb = Vec::with_capacity((width * height * 4) as usize);
    for pixel in icon.as_raw().chunks_exact(4) {
        argb.extend_from_slice(&[pixel[3], pixel[0], pixel[1], pixel[2]]);
    }
    (argb, width, height)
}

impl CFVigilTray {
    fn dispatch(&self, action: &'static str) {
        let app = self.app.clone();
        let dispatcher = app.clone();
        let _ = dispatcher.run_on_main_thread(move || match action {
            "open" => show_card(&app),
            "toggle" => {
                let visible = if crate::linux_surface::enabled() {
                    crate::linux_surface::visible()
                } else {
                    app.get_webview_window("main")
                        .and_then(|window| window.is_visible().ok())
                        .unwrap_or(false)
                };
                if visible { hide_card(app.clone()); }
                else { show_card(&app); }
            }
            "history" => show_window(&app, "history"),
            "settings" => show_window(&app, "settings"),
            "about" => show_window(&app, "about"),
            "quit" => app.exit(0),
            _ => {}
        });
    }
}

impl Tray for CFVigilTray {
    fn id(&self) -> String { "cfvigil".into() }
    fn title(&self) -> String { "CFVigil".into() }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let (data, width, height) = tray_argb();
        vec![Icon { width: width as i32, height: height as i32, data }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.dispatch("toggle");
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Show Card".into(),
                activate: Box::new(|tray: &mut Self| tray.dispatch("open")),
                ..Default::default()
            }.into(),
            StandardItem {
                label: "Event History".into(),
                activate: Box::new(|tray: &mut Self| tray.dispatch("history")),
                ..Default::default()
            }.into(),
            StandardItem {
                label: "Settings".into(),
                activate: Box::new(|tray: &mut Self| tray.dispatch("settings")),
                ..Default::default()
            }.into(),
            StandardItem {
                label: "About".into(),
                activate: Box::new(|tray: &mut Self| tray.dispatch("about")),
                ..Default::default()
            }.into(),
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|tray: &mut Self| tray.dispatch("quit")),
                ..Default::default()
            }.into(),
        ]
    }
}

pub fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handler = CFVigilTray { app: app.handle().clone() }.spawn()?;
    app.manage(handler);
    Ok(())
}
