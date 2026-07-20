mod config;

use eframe::egui;
use std::time::{Duration, SystemTime};

#[derive(Clone, PartialEq)]
enum DeviceStatus {
    Online,
    Offline,
    Connecting,
}

struct AutoSshApp {
    local_status: DeviceStatus,
    remote_status: DeviceStatus,
    auto_connect: bool,
    last_connection: Option<SystemTime>,
    last_connection_text: String,
    show_settings: bool,

    cfg: config::Config,
    save_result: Option<String>,
}

impl AutoSshApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = config::Config::load();
        let auto_connect = cfg.auto_connect;

        Self {
            local_status: DeviceStatus::Online,
            remote_status: DeviceStatus::Offline,
            auto_connect,
            last_connection: None,
            last_connection_text: String::from("Never"),
            show_settings: false,
            cfg,
            save_result: None,
        }
    }
}

impl eframe::App for AutoSshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));

        if !self.show_settings {
            self.render_dashboard(ctx);
        } else {
            self.render_settings(ctx);
        }
    }
}

impl AutoSshApp {
    fn render_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AutoSSH");
            ui.separator();
            ui.add_space(8.0);

            status_card(ui, "Local Device", &self.local_status);
            ui.add_space(4.0);
            status_card(ui, "Remote Device", &self.remote_status);
            ui.add_space(4.0);

            let auto_label = if self.auto_connect {
                "Auto-Connect: Enabled"
            } else {
                "Auto-Connect: Disabled"
            };
            ui.label(
                egui::RichText::new(auto_label)
                    .color(if self.auto_connect {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::GRAY
                    }),
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Last Connection:");
                ui.colored_label(
                    egui::Color32::LIGHT_GRAY,
                    &self.last_connection_text,
                );
            });

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("Connect")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .on_hover_text("Manually connect to remote device")
                    .clicked()
                {
                    self.remote_status = DeviceStatus::Connecting;
                }

                if ui
                    .button(
                        egui::RichText::new("Disconnect")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .on_hover_text("Disconnect from remote device")
                    .clicked()
                {
                    self.remote_status = DeviceStatus::Offline;
                    self.last_connection = None;
                    self.last_connection_text = String::from("Never");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new("\u{2699} Settings")
                                .size(16.0)
                                .color(egui::Color32::WHITE),
                        )
                        .on_hover_text("Open settings")
                        .clicked()
                    {
                        self.show_settings = true;
                        self.save_result = None;
                    }
                });
            });
        });
    }

    fn render_settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();
            ui.add_space(8.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Device Name:");
                    ui.text_edit_singleline(&mut self.cfg.device_name);
                    ui.end_row();

                    ui.label("Remote Host:");
                    ui.text_edit_singleline(&mut self.cfg.remote_host);
                    ui.end_row();

                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.cfg.username);
                    ui.end_row();

                    ui.label("SSH Port:");
                    ui.add(egui::DragValue::new(&mut self.cfg.port).range(1..=65535));
                    ui.end_row();

                    ui.label("SSH Key Path:");
                    ui.text_edit_singleline(&mut self.cfg.ssh_key_path);
                    ui.end_row();

                    ui.label("Poll Interval (s):");
                    ui.add(
                        egui::Slider::new(&mut self.cfg.poll_interval_seconds, 1..=60)
                            .integer(),
                    );
                    ui.end_row();

                    ui.label("Auto-Connect:");
                    ui.checkbox(&mut self.cfg.auto_connect, "");
                    ui.end_row();
                });

            ui.add_space(8.0);

            if let Some(ref result) = self.save_result {
                let (msg, color) = match result.as_str() {
                    "ok" => ("Config saved.", egui::Color32::GREEN),
                    _ => (result.as_str(), egui::Color32::RED),
                };
                ui.colored_label(color, msg);
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("Save")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .clicked()
                {
                    self.auto_connect = self.cfg.auto_connect;
                    match self.cfg.save() {
                        Ok(()) => {
                            self.save_result = Some(String::from("ok"));
                            log::info!("Configuration saved successfully");
                        }
                        Err(e) => {
                            self.save_result = Some(e);
                            log::error!("Failed to save configuration");
                        }
                    }
                }
                if ui
                    .button(
                        egui::RichText::new("Cancel")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .clicked()
                {
                    self.show_settings = false;
                }
            });
        });
    }
}

fn status_card(ui: &mut egui::Ui, label: &str, status: &DeviceStatus) {
    let (status_text, color) = match status {
        DeviceStatus::Online => ("Online", egui::Color32::GREEN),
        DeviceStatus::Offline => ("Offline", egui::Color32::RED),
        DeviceStatus::Connecting => ("Connecting...", egui::Color32::YELLOW),
    };

    egui::Frame::NONE
        .fill(egui::Color32::from_rgb(30, 30, 30))
        .corner_radius(4.0)
        .stroke(egui::epaint::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgb(60, 60, 60),
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(color, status_text);
                });
            });
        });
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 400.0])
            .with_min_inner_size([400.0, 350.0])
            .with_title("AutoSSH"),
        ..Default::default()
    };

    eframe::run_native(
        "AutoSSH",
        options,
        Box::new(|cc| Ok(Box::new(AutoSshApp::new(cc)))),
    )
}
