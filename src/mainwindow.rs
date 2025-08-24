use eframe::egui;

pub const CARD_WIDTH: f32 = 60.0;
pub const CARD_HEIGHT: f32 = 90.0;

#[derive(Clone)]
pub struct BrowserShow {
    pub name: String,
    pub path: String,
    pub cmd: Vec<String>,
    pub icon: egui::TextureId,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone)]
pub struct BrowserExec {
    pub path: String,
    pub cmd: Vec<String>,
}

impl BrowserShow {
    pub fn new(
        ctx: &egui::Context,
        name: String,
        path: String,
        cmd: Vec<String>,
        icon: image::RgbaImage,
    ) -> Self {
        let manager = ctx.tex_manager();
        let size = [icon.width() as _, icon.height() as _];
        let pixels = icon.as_flat_samples();
        let colorimg = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let icon = manager.write().alloc(
            path.clone(),
            colorimg.into(),
            egui::TextureOptions::default(),
        );
        Self {
            name,
            path,
            cmd,
            icon,
            width: size[0] as f32,
            height: size[1] as f32,
        }
    }
    pub fn new_without_icon(
        name: String,
        path: String,
        cmd: Vec<String>,
        default_icon: egui::TextureId,
        size: [usize; 2],
    ) -> Self {
        Self {
            name,
            path,
            cmd,
            icon: default_icon,
            width: size[0] as f32,
            height: size[1] as f32,
        }
    }
    pub fn into_exec(&self) -> BrowserExec {
        BrowserExec {
            path: self.path.clone(),
            cmd: self.cmd.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MainWindow {
    url: String,
    browsers: Vec<BrowserShow>,
    cols: usize,
}

impl MainWindow {
    pub fn new(url: String, browsers: Vec<BrowserShow>, cols: usize) -> Self {
        Self {
            url,
            browsers,
            cols,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let copyshortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);
        let exitshortcut_0 = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q);
        let exitshortcut_1 = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape);
        let open_shortcuts = [
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num1),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num2),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num3),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num4),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num5),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num6),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num7),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num8),
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Num9),
        ];
        let mut copy_cmd = false;
        let mut exit_cmd = false;
        let mut settings_cmd = false;
        let mut log_cmd = false;
        let mut open_browser = None;
        ctx.input_mut(|r| {
            copy_cmd = copy_cmd || r.consume_shortcut(&copyshortcut);
            exit_cmd = exit_cmd
                || r.consume_shortcut(&exitshortcut_0)
                || r.consume_shortcut(&exitshortcut_1);
            for i in 0..open_shortcuts.len() {
                if r.consume_shortcut(&open_shortcuts[i]) && i < self.browsers.len() {
                    open_browser = Some(self.browsers[i].into_exec());
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (drag_resp, _) = ui
                    .allocate_painter(egui::vec2(20.0, ui.available_height()), egui::Sense::drag());
                if drag_resp.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                exit_cmd = exit_cmd
                    || ui
                        .button("\u{ea76}")
                        .on_hover_text("Ctrl + Q\r\nESC")
                        .clicked();
                settings_cmd = ui.button("\u{eb52}").clicked();
                copy_cmd = copy_cmd || ui.button("\u{ebcc}").on_hover_text("Ctrl + S").clicked();
                log_cmd = ui.button("\u{f4ed}").clicked();
                ui.add(
                    egui::TextEdit::singleline(&mut self.url).desired_width(ui.available_width()),
                )
            });
            egui::ScrollArea::new([true, true])
                .max_width(ui.available_width())
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for i in 0..(self.browsers.len() / self.cols + 1) {
                        ui.horizontal(|ui| {
                            for j in 0..self.cols {
                                let index = i * self.cols + j;
                                if index < self.browsers.len() {
                                    let browser = &self.browsers[index];
                                    let mut cur = ui.cursor();
                                    cur.set_width(CARD_WIDTH);
                                    cur.set_height(CARD_HEIGHT);
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(cur), |ui| {
                                        ui.vertical(|ui| {
                                            if ui
                                                .add(egui::ImageButton::new(
                                                    egui::Image::from_texture(
                                                        egui::load::SizedTexture::new(
                                                            browser.icon,
                                                            [browser.width, browser.height],
                                                        ),
                                                    )
                                                    .fit_to_exact_size(egui::vec2(
                                                        CARD_WIDTH, CARD_WIDTH,
                                                    )),
                                                ))
                                                .clicked()
                                            {
                                                open_browser = Some(browser.into_exec());
                                            }
                                            ui.add_sized(
                                                egui::vec2(CARD_WIDTH, CARD_HEIGHT - CARD_WIDTH),
                                                egui::Label::new(&browser.name),
                                            );
                                        });
                                    });
                                } else {
                                    break;
                                }
                            }
                        });
                    }
                });
        });
        if copy_cmd {
            ctx.copy_text(self.url.clone());
        }
        if exit_cmd {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(exec) = open_browser {
            let mut cmds = exec.cmd.clone();
            cmds.push(self.url.clone());
            let result = std::process::Command::new(exec.path.clone())
                .args(cmds)
                .spawn();
            if let Err(e) = result {
                log::error!("Failed to open browser: {}", e);
            } else {
                log::info!(
                    "Opened browser: {:?}",
                    exec.path.clone() + " " + &exec.cmd.join(" ")
                );
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if settings_cmd {
            // use default text editor to open settings file
            let display_path = crate::utils::settings_path()
                .to_str()
                .unwrap_or_default()
                .to_string();
            crate::utils::open_file(&display_path);
        }
        if log_cmd {
            // use default text editor to open log file
            let display_path = crate::utils::log_path()
                .to_str()
                .unwrap_or_default()
                .to_string();
            crate::utils::open_file(&display_path);
        }
    }
}
