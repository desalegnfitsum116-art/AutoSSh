use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum SshCommand {
    Connect {
        host: String,
        port: u16,
        username: String,
        key_path: String,
    },
    Disconnect,
    Shutdown,
}

pub enum SshEvent {
    Connected,
    Disconnected(String),
    Error(String),
}

pub struct SshHandle {
    cmd_tx: mpsc::Sender<SshCommand>,
    event_rx: mpsc::Receiver<SshEvent>,
}

impl SshHandle {
    pub fn connect(&self, host: String, port: u16, username: String, key_path: String) {
        let _ = self
            .cmd_tx
            .send(SshCommand::Connect {
                host,
                port,
                username,
                key_path,
            });
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

pub fn start_ssh_manager() -> SshHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut session: Option<Session> = None;

        loop {
            match cmd_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(SshCommand::Connect {
                    host,
                    port,
                    username,
                    key_path,
                }) => {
                    if let Some(ref sess) = session {
                        if is_alive(sess) {
                            let _ = event_tx.send(SshEvent::Connected);
                            continue;
                        }
                    }

                    match create_session(&host, port, &username, &key_path) {
                        Ok(sess) => {
                            log::info!(
                                "SSH session established to {}@{}:{}",
                                username,
                                host,
                                port
                            );
                            session = Some(sess);
                            let _ = event_tx.send(SshEvent::Connected);
                        }
                        Err(e) => {
                            log::error!("SSH connection failed: {}", e);
                            let _ = event_tx.send(SshEvent::Error(format!(
                                "Connection failed: {}",
                                e
                            )));
                        }
                    }
                }
                Ok(SshCommand::Disconnect) => {
                    if let Some(sess) = session.take() {
                        let _ = sess.disconnect(None, "Client disconnect", None);
                        let _ = event_tx.send(SshEvent::Disconnected(
                            "Disconnected by user".into(),
                        ));
                        log::info!("SSH session disconnected by user");
                    }
                }
                Ok(SshCommand::Shutdown) => {
                    if let Some(sess) = session.take() {
                        let _ = sess.disconnect(None, "Shutdown", None);
                    }
                    let _ = event_tx.send(SshEvent::Disconnected("Shutdown".into()));
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(ref sess) = session {
                        if !is_alive(sess) {
                            log::warn!("SSH session appears dead, removing");
                            let _ = event_tx.send(SshEvent::Disconnected(
                                "Session lost".into(),
                            ));
                            session = None;
                        } else {
                            let _ = sess.keepalive_send();
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
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

    sess.set_keepalive(true, 15);
    sess.set_blocking(true);

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
