<div align="center">
  <h1>🔌 AutoSSH</h1>
  <p><strong>Automatic SSH connections that just work.</strong></p>
  <p>AutoSSH is a lightweight desktop utility that automatically connects trusted devices through SSH when both are powered on and reachable — no terminals, no manual commands.</p>
</div>

<p align="center">
  <img src="https://img.shields.io/badge/rust-stable-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License">
  <img src="https://img.shields.io/badge/platform-linux%20|%20windows%20|%20macos-lightgrey" alt="Cross Platform">
  <img src="https://img.shields.io/badge/build-passing-brightgreen" alt="Build Passing">
</p>

<br>

<!-- Banner placeholder -->
<p align="center">
  <i> <!-- Replace with actual screenshot -->
    [Screenshot of AutoSSH dashboard showing connected devices]
  </i>
</p>

---

## Features

- **Auto-connect** — Establishes an SSH session as soon as the remote device becomes reachable.
- **Auto-reconnect** — Survives network drops, sleep/wake cycles, and remote reboots with exponential backoff.
- **Zero terminal usage** — No need to open a shell and type SSH commands.
- **SSH key authentication** — Uses your existing `id_ed25519` or `id_rsa` keys.
- **Background service** — Runs quietly in the system tray and monitors devices 24/7.
- **Live status** — Dashboard shows: *Offline → Online → SSH Ready → Connected*.
- **System tray integration** — Connect, disconnect, or quit directly from the tray menu.
- **Native GUI** — Built with `egui`/`eframe` — lightweight, responsive, dark-themed.
- **Start on boot** — Optionally launch at login and minimize to tray.
- **Portable config** — Settings stored in `~/.config/auto-ssh/config.toml`.

---

## Why AutoSSH?

If you manage multiple computers — a laptop, a home server, a Raspberry Pi — you've probably written a shell script to SSH into them. AutoSSH does that for you automatically.

It feels like **Syncthing** or **AirDrop**: devices discover each other, connect, and stay connected. You don't think about it.

**Use cases:**

- A developer who wants to SSH into a home server without running `ssh user@host` every time.
- A homelab user who needs persistent terminal access to a Raspberry Pi.
- A DevOps engineer managing jump-box access.
- Anyone who has ever said *"Let me just SSH in… wait, is it on?"*

---

## Installation

### Linux

#### From .deb file
Go to the latest release in the release notes and download the .deb file and install it to your computer.

#### From .appimage file
Go to the latest release in the release notes and download the .appimage file and run the following commands
```bash
chmod +x auto-ssh-x86_64.AppImage
./auto-ssh-x86_64.AppImage 
```

#### From source (recommended)

```bash
git clone https://github.com/yourusername/autossh.git
cd autossh
cargo build --release
./target/release/auto-ssh
```

**Dependencies (Debian/Ubuntu):**

```bash
sudo apt install build-essential libssl-dev cmake
```

For the system tray, AutoSSH uses the [`ksni`](https://crates.io/crates/ksni) crate (pure Rust StatusNotifierItem), so no additional indicator libraries are required on modern Linux desktops.

#### Arch Linux

```bash
# AUR package coming soon
```

### macOS

```bash
cargo build --release
./target/release/auto-ssh
```

### Windows

```powershell
cargo build --release
.\target\release\auto-ssh.exe
```

---

## Quick Start

1. Launch AutoSSH.
2. Open **Settings** (gear icon).
3. Enter the remote host, username, SSH port, and path to your SSH private key.
4. Toggle **Auto-Connect** to *Enabled*.
5. Click **Save**.

AutoSSH will immediately start monitoring the remote device.

```
Device status cycle:

  Offline     → Host unreachable or DNS fails to resolve.
  Online      → Host resolves but SSH port is closed.
  SSH Ready   → Port 22 is open and accepting connections.
  Connected   → SSH session established and authenticated.
```

Once the remote device appears as **SSH Ready** (or better), AutoSSH connects automatically. The tray icon turns green and the dashboard shows **Connected**.

---

## How AutoConnect Works

```
┌─────────────────────────────────────────────────────┐
│                    AutoSSH Engine                    │
│                                                     │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐  │
│  │  Monitor  │───▶│   SSH    │───▶│  Keepalive   │  │
│  │  (tokio)  │    │ Manager  │    │  (15s tick)  │  │
│  └──────────┘    └──────────┘    └──────────────┘  │
│       │               │               │              │
│       ▼               ▼               ▼              │
│  DNS + TCP/22    Key Auth +      Session health     │
│  every 5s       Session create   check every 30s    │
│                                                     │
│  If session dies → exponential backoff reconnect     │
│  1s → 2s → 4s → 8s → 16s → 32s → 60s (capped)     │
└─────────────────────────────────────────────────────┘
```

### Rules

```
IF device is online
  AND SSH port (22) is reachable
  AND SSH key authenticates successfully
THEN connect automatically.

IF connection drops
THEN retry with exponential backoff.
```

---

## Configuration

AutoSSH stores its configuration at `~/.config/auto-ssh/config.toml`.

```toml
device_name = "Laptop"
remote_host = "192.168.1.10"
username = "fitsum"
port = 22
ssh_key_path = "~/.ssh/id_ed25519"
auto_connect = true
start_on_boot = true
poll_interval_seconds = 5
```

| Key | Default | Description |
|-----|---------|-------------|
| `device_name` | `My Laptop` | Display name for the local device |
| `remote_host` | `192.168.1.100` | IP address or hostname of the remote device |
| `username` | `user` | SSH login username |
| `port` | `22` | SSH port |
| `ssh_key_path` | `~/.ssh/id_ed25519` | Path to the SSH private key |
| `auto_connect` | `true` | Automatically connect when device is ready |
| `start_on_boot` | `false` | Launch AutoSSH at system login |
| `poll_interval_seconds` | `5` | Interval between network checks |

---

## Screenshots

<!-- Placeholder: Add actual screenshots here -->
```
Dashboard                          Settings
┌──────────────────────┐           ┌──────────────────────┐
│ AutoSSH              │           │ Settings             │
│ ──────────────────   │           │ ──────────────────   │
│ Local Device  Online │           │ Device Name: [   ]   │
│ Remote Device  SSH   │           │ Remote Host: [   ]   │
│ Auto-Connect: Enabled│           │ Username:    [   ]   │
│ Last: Just now       │           │ SSH Port:    [22 ]   │
│                      │           │ Key Path: [       ]  │
│ [Connect] [Disconnect]│           │ Poll: [5]s           │
│              [⚙]     │           │ Auto-Connect: [✔]    │
└──────────────────────┘           │ Start on Boot: [✔]   │
                                   │ [Save] [Cancel]      │
                                   └──────────────────────┘
```

---

## Building from Source

### Prerequisites

- **Rust** 1.70+ (stable)
- **Cargo**
- **CMake** (for building `libssh2` from source)
- **OpenSSL dev headers**

```bash
# Clone
git clone https://github.com/yourusername/autossh.git
cd autossh

# Debug build
cargo build

# Release build
cargo build --release

# Strip (Linux) for smaller binary
strip target/release/auto-ssh
# → ~17 MB binary
```

---

## Roadmap

- [ ] **Remote Actions** — Run commands or start services on connect.
- [ ] **Multiple devices** — Monitor and connect to several remotes simultaneously.
- [ ] **Connection history** — Log of connection events.
- [ ] **Notifications** — Desktop notifications on connect/disconnect.
- [ ] **Password/encrypted key support** — `ssh-agent` integration.
- [ ] **Port knocking** — Firewall-friendly connection sequences.
- [ ] **TUI mode** — Terminal UI for headless servers.

---

## FAQ

**Q: Do I need root / sudo permissions?**  
No. AutoSSH runs as a regular user and uses standard SSH keys.

**Q: Can it handle password-based auth?**  
Not currently. SSH key authentication is preferred and more secure.

**Q: What happens when my laptop goes to sleep?**  
AutoSSH detects the connection drop, enters reconnection mode, and automatically reconnects when the network is available again (exponential backoff).

**Q: Does it work with non-standard SSH ports?**  
Yes. Set the port in Settings (or directly in `config.toml`).

**Q: Is this a terminal emulator?**  
No. AutoSSH establishes SSH sessions in the background. It does not provide a terminal UI or shell.

**Q: Can I use it with SSH jump hosts?**  
Not yet. This is on the roadmap.

---

## Security Notes

- **SSH key authentication only** — No passwords are stored or transmitted.
- **Credentials stay local** — Configuration is stored in your home directory.
- **No network egress** — AutoSSH only connects to the hosts you configure.
- **Key permissions** — Ensure your SSH private key has `chmod 600` permissions.
- **Host key verification** — Future versions will include fingerprint verification.

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for the full guide.

**Quick links:**

- [Open an issue](https://github.com/yourusername/autossh/issues)
- [Start a discussion](https://github.com/yourusername/autossh/discussions)
- [Code of Conduct](./CONTRIBUTING.md#code-of-conduct)

---

## License

This project is licensed under the MIT License — see [LICENSE](./LICENSE) for details.

---

## Acknowledgements

- [egui](https://github.com/emilk/egui) — Immediate mode GUI library
- [eframe](https://github.com/emilk/egui) — egui framework
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs) — Rust SSH2 bindings
- [tokio](https://tokio.rs) — Async runtime
- [ksni](https://crates.io/crates/ksni) — Pure Rust StatusNotifierItem
- [Syncthing](https://syncthing.net) — Inspiration for "it just works" design
