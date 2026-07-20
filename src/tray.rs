use ksni::blocking::TrayMethods;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub enum TrayAction {
    Connect,
    Disconnect,
    Show,
}

pub struct TrayHandle {
    action_rx: mpsc::Receiver<TrayAction>,
    update_connected: Arc<Mutex<bool>>,
}

impl TrayHandle {
    pub fn try_recv_action(&self) -> Result<TrayAction, mpsc::TryRecvError> {
        self.action_rx.try_recv()
    }

    pub fn update_connection_status(&self, connected: bool) {
        if let Ok(mut c) = self.update_connected.lock() {
            *c = connected;
        }
    }
}

struct AutoSshTray {
    connected: Arc<Mutex<bool>>,
    action_tx: mpsc::Sender<TrayAction>,
}

impl ksni::Tray for AutoSshTray {
    fn id(&self) -> String {
        String::from("auto-ssh")
    }

    fn icon_name(&self) -> String {
        if let Ok(c) = self.connected.lock() {
            if *c {
                return String::from("network-server");
            }
        }
        String::from("network-offline")
    }

    fn title(&self) -> String {
        String::from("AutoSSH")
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let is_connected = self.connected.lock().map(|c| *c).unwrap_or(false);

        vec![
            StandardItem {
                label: if is_connected {
                    "Disconnect".into()
                } else {
                    "Connect".into()
                },
                icon_name: if is_connected {
                    "network-offline".into()
                } else {
                    "network-server".into()
                },
                activate: Box::new(|this: &mut Self| {
                    let action = if this.connected.lock().map(|c| *c).unwrap_or(false) {
                        TrayAction::Disconnect
                    } else {
                        TrayAction::Connect
                    };
                    let _ = this.action_tx.send(action);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Show Dashboard".into(),
                icon_name: "computer".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.action_tx.send(TrayAction::Show);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| {
                    let _ = std::thread::spawn(|| {
                        std::thread::sleep(Duration::from_millis(100));
                        std::process::exit(0);
                    });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn start_tray() -> Option<TrayHandle> {
    let (action_tx, action_rx) = mpsc::channel();
    let connected = Arc::new(Mutex::new(false));
    let connected_clone = connected.clone();

    let tray = AutoSshTray {
        connected: connected_clone,
        action_tx,
    };

    match tray.spawn() {
        Ok(handle) => {
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let _ = handle.update(|_| {});
                }
            });

            Some(TrayHandle {
                action_rx,
                update_connected: connected,
            })
        }
        Err(e) => {
            log::warn!("Failed to create system tray: {}", e);
            None
        }
    }
}
