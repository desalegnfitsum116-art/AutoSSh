use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const BACKOFF_INITIAL: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(60);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct ConnectionParams {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_path: String,
}

pub enum SshCommand {
    Connect(ConnectionParams),
    Disconnect,
    Shutdown,
}

pub enum SshEvent {
    Connected,
    Connecting { attempt: u32, delay_secs: u64 },
    Disconnected(String),
}

pub struct SshHandle {
    cmd_tx: mpsc::Sender<SshCommand>,
    event_rx: mpsc::Receiver<SshEvent>,
}

impl SshHandle {
    pub fn connect(&self, params: ConnectionParams) {
        let _ = self.cmd_tx.send(SshCommand::Connect(params));
    }

    pub fn disconnect(&self) {
        let _ = self.cmd_tx.send(SshCommand::Disconnect);
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(SshCommand::Shutdown);
    }

    pub fn try_recv_event(&self) -> Result<SshEvent, mpsc::TryRecvError> {
        self.event_rx.try_recv()
    }
}

#[derive(Clone, PartialEq)]
enum EngineState {
    Idle,
    Connecting,
    Connected,
    Reconnecting,
}

struct Engine {
    session: Option<Session>,
    params: Option<ConnectionParams>,
    state: EngineState,
    auto_reconnect: bool,
    attempt: u32,
    backoff: Duration,
    last_action: Instant,
    last_health_check: Instant,
    cmd_rx: mpsc::Receiver<SshCommand>,
    event_tx: mpsc::Sender<SshEvent>,
}

impl Engine {
    fn new(cmd_rx: mpsc::Receiver<SshCommand>, event_tx: mpsc::Sender<SshEvent>) -> Self {
        Self {
            session: None,
            params: None,
            state: EngineState::Idle,
            auto_reconnect: true,
            attempt: 0,
            backoff: BACKOFF_INITIAL,
            last_action: Instant::now(),
            last_health_check: Instant::now(),
            cmd_rx,
            event_tx,
        }
    }

    fn emit(&self, event: SshEvent) {
        let _ = self.event_tx.send(event);
    }

    fn handle_command(&mut self, cmd: SshCommand) {
        match cmd {
            SshCommand::Connect(params) => {
                self.params = Some(params.clone());
                self.attempt = 1;
                self.backoff = BACKOFF_INITIAL;
                self.state = EngineState::Connecting;
                self.last_action = Instant::now();
                self.emit(SshEvent::Connecting {
                    attempt: 1,
                    delay_secs: 0,
                });
            }
            SshCommand::Disconnect => {
                self.disconnect_session("Disconnected by user");
                self.state = EngineState::Idle;
                self.emit(SshEvent::Disconnected("Disconnected by user".into()));
            }
            SshCommand::Shutdown => {
                self.disconnect_session("Shutdown");
                self.state = EngineState::Idle;
            }
        }
    }

    fn disconnect_session(&mut self, reason: &str) {
        if let Some(sess) = self.session.take() {
            let _ = sess.disconnect(None, reason, None);
            log::info!("SSH session disconnected: {}", reason);
        }
    }

    fn try_connect(&mut self) {
        let params = match &self.params {
            Some(p) => p.clone(),
            None => return,
        };

        self.emit(SshEvent::Connecting {
            attempt: self.attempt,
            delay_secs: self.backoff.as_secs(),
        });

        match create_session(&params.host, params.port, &params.username, &params.key_path) {
            Ok(session) => {
                log::info!(
                    "SSH session established to {}@{}:{} (attempt {})",
                    params.username,
                    params.host,
                    params.port,
                    self.attempt
                );
                self.session = Some(session);
                self.state = EngineState::Connected;
                self.last_action = Instant::now();
                self.last_health_check = Instant::now();
                self.attempt = 0;
                self.emit(SshEvent::Connected);
            }
            Err(e) => {
                log::warn!("Connection attempt {} failed: {}", self.attempt, e);
                self.attempt += 1;
                self.backoff = next_backoff(self.backoff);
                self.last_action = Instant::now();
            }
        }
    }

    fn check_health(&mut self) {
        if let Some(ref session) = self.session {
            if !is_alive(session) {
                log::warn!("SSH session dead, entering reconnection");
                self.disconnect_session("Session lost");
                self.emit(SshEvent::Disconnected("Session lost".into()));

                if self.auto_reconnect && self.params.is_some() {
                    self.state = EngineState::Reconnecting;
                    self.attempt = 1;
                    self.backoff = BACKOFF_INITIAL;
                    self.last_action = Instant::now();
                } else {
                    self.state = EngineState::Idle;
                }
            } else {
                self.last_health_check = Instant::now();
            }
        }
    }

    fn send_keepalive(&mut self) {
        if let Some(ref session) = self.session {
            let _ = session.keepalive_send();
            self.last_action = Instant::now();
        }
    }

    fn run(&mut self) {
        loop {
            let timeout = self.next_timeout();
            let cmd = if timeout == Duration::ZERO {
                self.cmd_rx.try_recv().ok()
            } else {
                match self.cmd_rx.recv_timeout(timeout) {
                    Ok(cmd) => Some(cmd),
                    Err(mpsc::RecvTimeoutError::Timeout) => None,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            };

            if let Some(cmd) = cmd {
                let is_shutdown = matches!(cmd, SshCommand::Shutdown);
                self.handle_command(cmd);
                if is_shutdown {
                    break;
                }
            }

            match self.state {
                EngineState::Connecting | EngineState::Reconnecting => {
                    if self.last_action.elapsed() >= self.backoff {
                        self.try_connect();
                    }
                }
                EngineState::Connected => {
                    let now = Instant::now();
                    if now.saturating_duration_since(self.last_health_check)
                        >= HEALTH_CHECK_INTERVAL
                    {
                        self.check_health();
                    }
                    if now.saturating_duration_since(self.last_action) >= KEEPALIVE_INTERVAL {
                        self.send_keepalive();
                    }
                }
                EngineState::Idle => {}
            }
        }
    }

    fn next_timeout(&self) -> Duration {
        match self.state {
            EngineState::Idle => Duration::from_millis(500),
            EngineState::Connecting | EngineState::Reconnecting => {
                let elapsed = self.last_action.elapsed();
                if elapsed >= self.backoff {
                    Duration::ZERO
                } else {
                    self.backoff.saturating_sub(elapsed)
                }
            }
            EngineState::Connected => {
                let now = Instant::now();
                let next_health = HEALTH_CHECK_INTERVAL
                    .saturating_sub(now.saturating_duration_since(self.last_health_check));
                let next_keepalive = KEEPALIVE_INTERVAL
                    .saturating_sub(now.saturating_duration_since(self.last_action));
                next_health.min(next_keepalive).max(Duration::from_millis(100))
            }
        }
    }
}

fn next_backoff(current: Duration) -> Duration {
    current.checked_mul(2).unwrap_or(BACKOFF_MAX).min(BACKOFF_MAX)
}

pub fn start_ssh_manager() -> SshHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut engine = Engine::new(cmd_rx, event_tx);
        engine.run();
    });

    SshHandle { cmd_tx, event_rx }
}

fn create_session(
    host: &str,
    port: u16,
    username: &str,
    key_path: &str,
) -> Result<Session, String> {
    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| format!("TCP connect to {} failed: {}", addr, e))?;
    tcp.set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create SSH session: {}", e))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    let expanded_key = shellexpand::tilde(key_path).to_string();
    let key_path = Path::new(&expanded_key);

    if !key_path.exists() {
        return Err(format!("SSH key not found: {}", expanded_key));
    }

    sess.userauth_pubkey_file(username, None, key_path, None)
        .map_err(|e| format!("SSH key authentication failed: {}", e))?;

    if !sess.authenticated() {
        return Err("SSH authentication not confirmed".into());
    }

    sess.set_timeout(10000);
    Ok(sess)
}

fn is_alive(sess: &Session) -> bool {
    let mut buf = [0u8; 1];
    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(_) => return false,
    };

    if channel.exec("echo alive").is_err() {
        return false;
    }

    let _ = channel.read(&mut buf);
    let _ = channel.wait_close();
    true
}
