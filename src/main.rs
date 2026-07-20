mod config;
mod monitor;
mod ssh;

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

struct AutoSshApp {
    local_status: monitor::DeviceState,
    remote_status: monitor::DeviceState,
    auto_connect: bool,
    last_connection: Option<SystemTime>,
    last_connection_text: String,
    show_settings: bool,

    cfg: config::Config,
    save_result: Option<String>,

    monitor_handle: Option<monitor::MonitorHandle>,
    ssh_handle: Option<ssh::SshHandle>,
    connected: Arc<Mutex<bool>>,
    auto_connect_attempted: bool,
}

impl AutoSshApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = config::Config::load();
        let auto_connect = cfg.auto_connect;
        let connected = Arc::new(Mutex::new(false));
        let connected_clone = connected.clone();

        let monitor_handle = Some(monitor::start_monitor(
            cfg.remote_host.clone(),
            cfg.port,
            cfg.poll_interval_seconds,
            connected_clone,
        ));

        let ssh_handle = Some(ssh::start_ssh_manager());

        Self {
            local_status: monitor::DeviceState::Online,
            remote_status: monitor::DeviceState::Offline,
            auto_connect,
            last_connection: None,
            last_connection_text: String::from("Never"),
            show_settings: false,
            cfg,
            save_result: None,
            monitor_handle,
            ssh_handle,
            connected,
            auto_connect_attempted: false,
        }
    }
}

impl eframe::App for AutoSshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));

        self.poll_monitor();
        self.poll_ssh_events();

        if self.auto_connect
            && !*self.connected.lock().unwrap()
            && !self.auto_connect_attempted
            && self.remote_status == monitor::DeviceState::SshReady
        {
            self.initiate_connection();
        }

        if !self.show_settings {
            self.render_dashboard(ctx);
        } else {
            self.render_settings(ctx);
        }
    }
}

impl Drop for AutoSshApp {
    fn drop(&mut self) {
        if let Some(ref handle) = self.monitor_handle {
            handle.shutdown();
        }
        if let Some(ref handle) = self.ssh_handle {
            handle.shutdown();
        }
    }
}

impl AutoSshApp {
    fn poll_monitor(&mut self) {
        if let Some(ref handle) = self.monitor_handle {
            while let Ok(state) = handle.recv_state() {
                self.remote_status = state;
            }
        }
    }

    fn poll_ssh_events(&mut self) {
        if let Some(ref handle) = self.ssh_handle {
            while let Ok(event) = handle.try_recv_event() {
                match event {
                    ssh::SshEvent::Connected => {
                        *self.connected.lock().unwrap() = true;
                        self.auto_connect_attempted = false;
                        self.remote_status = monitor::DeviceState::Connected;
                        self.last_connection = Some(SystemTime::now());
                        self.last_connection_text = String::from("Just now");
                        log::info!("SSH connection established");
                    }
                    ssh::SshEvent::Connecting { attempt, delay_secs } => {
                        self.auto_connect_attempted = true;
                        self.remote_status = monitor::DeviceState::Connected;
                        log::info!(
                            "SSH connecting (attempt {}, delay {}s)",
                            attempt,
                            delay_secs
                        );
                    }
                    ssh::SshEvent::Disconnected(reason) => {
                        *self.connected.lock().unwrap() = false;
                        self.auto_connect_attempted = false;
                        if self.remote_status == monitor::DeviceState::Connected {
                            self.remote_status = monitor::DeviceState::SshReady;
                            self.last_connection_text = String::from("Lost connection");
                        }
                        log::info!("SSH disconnected: {}", reason);
                    }

                }
            }
        }
    }

    fn initiate_connection(&mut self) {
        self.auto_connect_attempted = true;
        self.remote_status = monitor::DeviceState::Connected;

        if let Some(ref handle) = self.ssh_handle {
            handle.connect(ssh::ConnectionParams {
                host: self.cfg.remote_host.clone(),
                port: self.cfg.port,
                username: self.cfg.username.clone(),
                key_path: self.cfg.ssh_key_path.clone(),
            });
        }
        log::info!("Auto-connecting to {}:{}", self.cfg.remote_host, self.cfg.port);
    }

    fn render_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AutoSSH");
            ui.separator();
            ui.add_space(8.0);

            status_card(ui, "Local Device", self.local_status.label(), true);
            ui.add_space(4.0);
            status_card(ui, "Remote Device", self.remote_status.label(), false);
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

            let is_connected = *self.connected.lock().unwrap();
            let is_ready = self.remote_status == monitor::DeviceState::SshReady;

            ui.horizontal(|ui| {
                let connect_enabled = is_ready && !is_connected;
                if ui
                    .add_enabled(
                        connect_enabled,
                        egui::Button::new(
                            egui::RichText::new("Connect")
                                .size(16.0)
                                .color(egui::Color32::WHITE),
                        ),
                    )
                    .on_hover_text("Manually connect to remote device")
                    .clicked()
                {
                    if let Some(ref handle) = self.ssh_handle {
                        handle.connect(ssh::ConnectionParams {
                            host: self.cfg.remote_host.clone(),
                            port: self.cfg.port,
                            username: self.cfg.username.clone(),
                            key_path: self.cfg.ssh_key_path.clone(),
                        });
                    }
                }

                if ui
                    .add_enabled(
                        is_connected,
                        egui::Button::new(
                            egui::RichText::new("Disconnect")
                                .size(16.0)
                                .color(egui::Color32::WHITE),
                        ),
                    )
                    .on_hover_text("Disconnect from remote device")
                    .clicked()
                {
                    if let Some(ref handle) = self.ssh_handle {
                        handle.disconnect();
                    }
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
                            if let Some(ref handle) = self.monitor_handle {
                                handle.update_host(
                                    self.cfg.remote_host.clone(),
                                    self.cfg.port,
                                );
                            }
                            self.auto_connect_attempted = false;
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

fn status_card(ui: &mut egui::Ui, label: &str, status: &str, is_local: bool) {
    let color = if is_local {
        egui::Color32::GREEN
    } else {
        match status {
            "Offline" => egui::Color32::RED,
            "Online" => egui::Color32::YELLOW,
            "SSH Ready" => egui::Color32::from_rgb(0, 200, 255),
            "Connected" => egui::Color32::GREEN,
            _ => egui::Color32::GRAY,
        }
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
                    ui.colored_label(color, status);
                });
            });
        });
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 420.0])
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
