#![windows_subsystem = "windows"]

mod cache;
mod mainwindow;
mod settings;
pub mod utils;

use eframe::egui;
use log::{self, error, info, warn, LevelFilter};
use log4rs;
use std::io::{Error, ErrorKind};
use std::result::Result;

// Default placeholder icon used when browser icons can't be loaded
lazy_static::lazy_static! {
    pub static ref PLACEHOLDER: (egui::ColorImage, [usize; 2]) = {
        let icon = image::load_from_memory(include_bytes!("../assets/icon_main.png"))
            .expect("Failed to load placeholder icon")
            .to_rgba8();
        let size = [icon.width() as _, icon.height() as _];
        let pixels = icon.as_flat_samples();
        (egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()), size)
    };
}

fn main() -> Result<(), Error> {
    // Parse command line arguments
    let url = parse_command_line();

    // Initialize logger
    setup_logger()?;
    info!("Opening URL: {url}");

    // Load settings
    let settings = load_settings()?;

    // Calculate window dimensions
    let (inner_width, inner_height, pos_x, pos_y) = calculate_window_dimensions(&settings);

    // Create window options
    let options = create_window_options(inner_width, inner_height, pos_x, pos_y)?;

    // Store settings values we need for the app (move Copy types directly)
    let settings_browsers = settings.browsers.clone();
    let cache_exp_days = settings.cache_expire_days;
    let settings_cols = settings.cols;

    // Run the application
    eframe::run_native(
        "URL Proxy",
        options,
        Box::new(move |cc| {
            // Setup fonts
            setup_fonts(cc);

            // Setup browser icons
            let browsers = setup_browser_icons(cc, settings_browsers, cache_exp_days);

            // Set dark theme
            cc.egui_ctx.set_theme(egui::Theme::Dark);

            // Create and return the main window
            Ok(Box::new(mainwindow::MainWindow::new(
                url,
                browsers,
                settings_cols,
            )))
        }),
    )
    .map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to run application: {}", e),
        )
    })?;

    Ok(())
}

/// Parse command line arguments and return the URL
fn parse_command_line() -> String {
    std::env::args().nth(1).unwrap_or_default()
}

/// Setup the logger with rolling file appender
fn setup_logger() -> Result<(), Error> {
    let file_roller = log4rs::append::rolling_file::RollingFileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("[{d} - {l}] {m}{n}")))
        .build(
            utils::log_path(),
            Box::new(log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
                Box::new(log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger::new(512 * 1024)),
                Box::new(log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller::new())
            ))
        )
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create log file appender: {}", e)))?;

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("main", Box::new(file_roller)))
        .build(
            log4rs::config::Root::builder()
                .appender("main")
                .build(LevelFilter::Info),
        )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to build logger config: {}", e),
            )
        })?;

    log4rs::init_config(config).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to initialize logger: {}", e),
        )
    })?;

    Ok(())
}

/// Load application settings or create default ones
fn load_settings() -> Result<settings::Settings, Error> {
    let settings_path = utils::settings_path();

    let settings = if settings_path.exists() {
        if settings_path.is_file() {
            settings::Settings::load()
        } else {
            error!("Settings {} is not a file.", settings_path.display());
            settings::Settings::default()
        }
    } else {
        warn!("Settings {} does not exist.", settings_path.display());
        let settings = settings::Settings::default();
        if let Err(e) = settings.create() {
            error!("Failed to create settings file: {e}");
        }
        settings
    };

    Ok(settings)
}

/// Calculate window dimensions based on settings and screen size
fn calculate_window_dimensions(settings: &settings::Settings) -> (f32, f32, f32, f32) {
    let (sc_width, sc_height) = utils::get_screen_size();

    // Calculate window size
    let inner_width = settings.cols as f32 * (mainwindow::CARD_WIDTH + 20.0);
    let inner_height = settings.rows as f32 * (mainwindow::CARD_HEIGHT + 15.0) + 40.0;

    // Calculate window position (centered)
    let pos_x = (sc_width as f32 - inner_width) / 2.0;
    let pos_y = (sc_height as f32 - inner_height) / 2.0;

    (inner_width, inner_height, pos_x, pos_y)
}

/// Create window options for eframe
fn create_window_options(
    inner_width: f32,
    inner_height: f32,
    pos_x: f32,
    pos_y: f32,
) -> Result<eframe::NativeOptions, Error> {
    // Load application icon
    let icon_img =
        image::load_from_memory(include_bytes!("../assets/icon_main.png")).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to load application icon: {}", e),
            )
        })?;
    let icon_buffer = icon_img.to_rgba8();
    let icon_pixels = icon_buffer.as_flat_samples();

    let icon_data = egui::IconData {
        rgba: icon_pixels.to_vec().samples,
        width: icon_img.width(),
        height: icon_img.height(),
    };

    // Create window options
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            inner_size: Some(egui::vec2(inner_width, inner_height)),
            position: Some(egui::pos2(pos_x, pos_y)),
            window_level: Some(egui::WindowLevel::AlwaysOnTop),
            resizable: Some(false),
            decorations: Some(false),
            icon: Some(std::sync::Arc::new(icon_data)),
            ..Default::default()
        },
        ..Default::default()
    };

    Ok(options)
}

/// Setup fonts for the application
fn setup_fonts(cc: &eframe::CreationContext) {
    let mut fonts = egui::FontDefinitions::default();

    // Add custom fonts
    fonts.font_data.insert(
        "fonts".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/Roboto-Regular.ttf")),
    );
    fonts.font_data.insert(
        "symbols".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/SymbolsNerdFontMono-Regular.ttf")),
    );

    // Configure font families
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

    // Apply fonts
    cc.egui_ctx.set_fonts(fonts);
}

/// Setup browser icons and create browser objects
fn setup_browser_icons(
    cc: &eframe::CreationContext,
    browsers_config: Vec<settings::BrowserInfo>,
    cache_expire_days: usize,
) -> Vec<mainwindow::BrowserShow> {
    // Create texture manager and default icon
    let manager = cc.egui_ctx.tex_manager();
    let default_icon = manager.write().alloc(
        "PLACEHOLDER".into(),
        PLACEHOLDER.0.clone().into(),
        egui::TextureOptions::default(),
    );

    // Initialize cache manager
    let mut cache = cache::IconCacheManager::new(cache_expire_days);
    let mut browsers = Vec::new();

    // Create browser objects with icons
    for browser in browsers_config {
        let icon = cache.get(&browser.name, &browser.path);
        let browser_show = match icon {
            Some(img) => mainwindow::BrowserShow::new(
                &cc.egui_ctx,
                browser.name,
                browser.path,
                browser.cmd,
                img,
            ),
            None => mainwindow::BrowserShow::new_without_icon(
                browser.name,
                browser.path,
                browser.cmd,
                default_icon,
                PLACEHOLDER.1,
            ),
        };
        browsers.push(browser_show);
    }

    browsers
}
