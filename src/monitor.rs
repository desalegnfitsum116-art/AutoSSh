use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceState {
    Offline,
    Online,
    SshReady,
    Connected,
}

impl DeviceState {
    pub fn label(&self) -> &str {
        match self {
            DeviceState::Offline => "Offline",
            DeviceState::Online => "Online",
            DeviceState::SshReady => "SSH Ready",
            DeviceState::Connected => "Connected",
        }
    }
}

pub struct MonitorHandle {
    state_rx: mpsc::Receiver<DeviceState>,
    command_tx: mpsc::Sender<MonitorCommand>,
}

enum MonitorCommand {
    UpdateConfig(String, u16),
    Shutdown,
}

impl MonitorHandle {
    pub fn recv_state(&self) -> Result<DeviceState, mpsc::TryRecvError> {
        self.state_rx.try_recv()
    }

    pub fn update_host(&self, host: String, port: u16) {
        let _ = self.command_tx.send(MonitorCommand::UpdateConfig(host, port));
    }

    pub fn shutdown(&self) {
        let _ = self.command_tx.send(MonitorCommand::Shutdown);
    }
}

struct SharedState {
    host: String,
    port: u16,
    poll_interval: u64,
}

pub fn start_monitor(
    initial_host: String,
    initial_port: u16,
    poll_interval: u64,
    connected: Arc<Mutex<bool>>,
) -> MonitorHandle {
    let (state_tx, state_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel();

    let shared = Arc::new(Mutex::new(SharedState {
        host: initial_host,
        port: initial_port,
        poll_interval,
    }));

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .expect("Failed to build tokio runtime");

        rt.block_on(async move {
            loop {
                let (host, port) = {
                    let s = shared.lock().unwrap();
                    (s.host.clone(), s.port)
                };

                let is_connected = *connected.lock().unwrap();
                if is_connected {
                    let _ = state_tx.send(DeviceState::Connected);
                } else {
                    let state = check_device(&host, port, &connected).await;
                    let _ = state_tx.send(state);
                }

                let poll_secs = {
                    let s = shared.lock().unwrap();
                    s.poll_interval
                };

                let cmd = cmd_rx.recv_timeout(Duration::from_secs(poll_secs));
                match cmd {
                    Ok(MonitorCommand::UpdateConfig(h, p)) => {
                        let mut s = shared.lock().unwrap();
                        s.host = h;
                        s.port = p;
                    }
                    Ok(MonitorCommand::Shutdown) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
    });

    MonitorHandle {
        state_rx,
        command_tx: cmd_tx,
    }
}

async fn check_device(host: &str, port: u16, connected: &Arc<Mutex<bool>>) -> DeviceState {
    if *connected.lock().unwrap() {
        return DeviceState::Connected;
    }

    let addr = match resolve_host(host).await {
        Some(a) => a,
        None => return DeviceState::Offline,
    };

    if check_port(addr, port).await {
        DeviceState::SshReady
    } else {
        DeviceState::Online
    }
}

async fn resolve_host(host: &str) -> Option<SocketAddr> {
    match tokio::net::lookup_host((host, 0)).await {
        Ok(mut addrs) => addrs.next(),
        Err(_) => None,
    }
}

async fn check_port(addr: SocketAddr, port: u16) -> bool {
    let target = SocketAddr::new(addr.ip(), port);
    match timeout(Duration::from_secs(3), TcpStream::connect(target)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}
