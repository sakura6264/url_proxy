#![windows_subsystem = "windows"]

mod cache;
mod mainwindow;
mod settings;
pub mod utils;
use eframe::egui;

lazy_static::lazy_static! {
    pub static ref PLACEHOLDER: (egui::ColorImage, [usize; 2]) = {
        let icon = image::load_from_memory(include_bytes!("../assets/icon_main.png")).unwrap().to_rgba8();
        let size = [icon.width() as _, icon.height() as _];
        let pixels = icon.as_flat_samples();
        (egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()), size)

    };
}

fn main() {
    // set environment variable DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1=1 to avoid crash on AMD
    std::env::set_var("DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1", "1");
    // parse command line arguments
    let args = std::env::args().collect::<Vec<String>>();
    let url;
    if args.len() < 2 {
        url = String::new();
    } else {
        url = args[1].clone();
    }
    // initialize logger
    let file_roller = log4rs::append::rolling_file::RollingFileAppender::builder()
    .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("[{d} - {l}] {m}{n}")))
    .build(crate::utils::log_path(), Box::new(
        log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
            Box::new(log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger::new(512 * 1024)),
            Box::new(log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller::new())
    ))).unwrap();
    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("main", Box::new(file_roller)))
        .build(
            log4rs::config::Root::builder()
                .appender("main")
                .build(log::LevelFilter::Info),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();
    log::info!("Opening URL:{url}");
    // use winit to read screen size
    // load settings
    let (sc_width, sc_height) = utils::get_screen_size();
    let settings = settings::Settings::load();
    // calculate window size
    let inner_width = settings.cols as f32 * (mainwindow::CARD_WIDTH + 20.0);
    let inner_height = settings.rows as f32 * (mainwindow::CARD_HEIGHT + 15.0) + 40.0;
    // calculate windows position
    let pos_x = (sc_width as f32 - inner_width) / 2.0;
    let pos_y = (sc_height as f32 - inner_height) / 2.0;
    let icon_img = image::load_from_memory(include_bytes!("../assets/icon_main.png")).unwrap();
    let icon_buffer = icon_img.to_rgba8();
    let icon_pixels = icon_buffer.as_flat_samples();
    let icon_data = egui::IconData {
        rgba: icon_pixels.to_vec().samples,
        width: icon_img.width(),
        height: icon_img.height(),
    };
    let option = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            inner_size: Some(egui::vec2(inner_width, inner_height)),
            position: Some(egui::pos2(pos_x, pos_y)),
            window_level: Some(egui::WindowLevel::AlwaysOnTop),
            resizable: Some(false),
            decorations: Some(false),
            icon: Some(std::sync::Arc::new(icon_data)),
            ..Default::default()
        },
        default_theme: eframe::Theme::Dark,
        follow_system_theme: false,
        ..Default::default()
    };
    let settings_browsers = settings.browsers.clone();
    let cache_exp_days = Box::leak(Box::new(settings.cache_expire_days));
    let settings_cols = Box::leak(Box::new(settings.cols));
    eframe::run_native(
        "URL Proxy",
        option,
        Box::new(|cc| {
            // set fonts
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "fonts".to_string(),
                egui::FontData::from_static(include_bytes!("../assets/Roboto-Regular.ttf")),
            );
            fonts.font_data.insert(
                "symbols".to_string(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/SymbolsNerdFontMono-Regular.ttf"
                )),
            );
            let proportional = fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default();
            proportional.insert(0, "fonts".to_string());
            proportional.insert(1, "symbols".to_string());

            let monospace = fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default();
            monospace.insert(0, "fonts".to_string());
            monospace.insert(1, "symbols".to_string());
            cc.egui_ctx.set_fonts(fonts);
            let manager = cc.egui_ctx.tex_manager();
            let default_icon = manager.write().alloc(
                "PLACEHOLDER".into(),
                PLACEHOLDER.0.clone().into(),
                egui::TextureOptions::default(),
            );
            let mut browsers = Vec::new();
            let mut cache = cache::IconCacheManager::new(*cache_exp_days);

            for b in settings_browsers {
                let icon = cache.get(&b.name, &b.path);
                let bros = match icon {
                    Some(img) => {
                        mainwindow::BrowserShow::new(&cc.egui_ctx, b.name, b.path, b.cmd, img)
                    }
                    None => mainwindow::BrowserShow::new_without_icon(
                        b.name,
                        b.path,
                        b.cmd,
                        default_icon,
                        PLACEHOLDER.1,
                    ),
                };
                browsers.push(bros);
            }
            Box::new(mainwindow::MainWindow::new(url, browsers, *settings_cols))
        }),
    )
    .unwrap();
    if let Err(e) = settings.save() {
        log::error!("Save settings failed:{e}");
    }
}
