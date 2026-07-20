# Contributing to AutoSSH

Hey, thanks for stopping by. 👋

AutoSSH is a small utility built by people who want SSH connections to be boring — set once, connect forever. Whether you're fixing a bug, adding a feature, improving docs, or just asking a question, you're welcome here.

---

## Code of Conduct

This project follows a simple rule: **be excellent to each other**. We expect all contributors to foster a respectful and inclusive environment. Harassment, trolling, and personal attacks are not tolerated.

If you experience or witness unacceptable behavior, please open an issue or contact the maintainers directly.

---

## Getting Started

### Required Tools

- **Rust** 1.70+ (stable toolchain)
- **Cargo** (comes with Rust)
- **Git**
- **CMake** (for building `libssh2` from source)

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

### Clone and Build

```bash
git clone https://github.com/yourusername/autossh.git
cd autossh
cargo build
```

---

## Development Workflow

### Running the Project

```bash
# Debug build with hot-reload-like iteration
cargo run

# Release build
cargo run --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Formatting

Format all code with `rustfmt` before committing:

```bash
cargo fmt
```

### Linting

Run Clippy to catch common mistakes and enforce idioms:

```bash
cargo clippy --all-targets --all-features
```

Aim for zero warnings. If you must suppress a lint, add a comment explaining why.

### Checking for Errors

```bash
cargo check
```

This is faster than a full build and catches type errors and borrow-checker issues.

---

## Project Structure

```
src/
├── main.rs         — App entry, egui dashboard, settings UI
├── config.rs       — TOML config load/save (XDG config directory)
├── monitor.rs      — Background network poller (tokio)
├── ssh.rs          — SSH session manager, reconnection engine
├── tray.rs         — System tray (ksni / StatusNotifierItem)
└── autostart.rs    — Desktop autostart file management
```

### Module responsibilities

| Module | Role |
|--------|------|
| `config` | Read/write `~/.config/auto-ssh/config.toml` |
| `monitor` | Async tokio thread that resolves DNS + checks TCP port every N seconds |
| `ssh` | State machine: Idle → Connecting → Connected → Reconnecting; exponential backoff |
| `tray` | StatusNotifierItem via DBus; menu actions relayed via mpsc channel |
| `autostart` | Write/remove `~/.config/autostart/auto-ssh.desktop` |

---

## Branch Naming

Use descriptive names with a forward slash separator:

```
feature/multiple-devices
fix/tray-icon-crash
docs/readme-typo
refactor/monitor-module
```

---

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add automatic reconnection with exponential backoff
fix: handle SSH key path with tilde expansion
docs: update configuration example in README
refactor: extract network check into separate module
chore: bump tokio dependency to 1.40
```

Keep the first line under 72 characters. Add a body if the change needs explanation:

```
feat: add start-on-boot toggle in settings

Creates a .desktop file in XDG autostart directory when the user
enables "Start on Boot" in Settings → Save. Removes the file when
the setting is disabled.
```

---

## Pull Request Checklist

Before opening a PR:

- [ ] `cargo fmt` has been run
- [ ] `cargo clippy --all-targets --all-features` passes with no warnings
- [ ] `cargo test` passes
- [ ] `cargo build --release` compiles successfully
- [ ] New code includes appropriate error handling and logging (`log::info!`, `log::warn!`, `log::error!`)
- [ ] Changes are scoped to a single concern (split into multiple PRs if needed)
- [ ] Commit messages are clear and follow the convention

---

## Reporting Bugs

Open an issue with the following information:

1. **Environment**: OS, desktop environment (GNOME / KDE / etc.), Rust version
2. **Steps to reproduce**: What did you do?
3. **Expected behavior**: What should happen?
4. **Actual behavior**: What happened instead?
5. **Logs**: Run with `RUST_LOG=info cargo run` and include the output
6. **Config**: Sanitized contents of `~/.config/auto-ssh/config.toml`

---

## Suggesting Features

Feature requests are welcome. Open an issue or start a Discussion.

When suggesting, include:

- What problem you're solving
- How you envision the feature working
- Whether you'd be willing to help implement it

Small, focused features have a better chance of being reviewed quickly.

---

## Documentation

Docs improvements are incredibly valuable. If you see a typo, unclear section, or missing example:

- Fix it in a README or doc comment and open a PR.
- Add doc comments (`///`) to public functions and types.
- Update the README if your change affects the user-facing behavior.

---

## First-Time Contributors

If this is your first open-source contribution, welcome! Here are some good starting points:

- Add a doc comment to an undocumented function.
- Improve error messages in `src/ssh.rs`.
- Write a test for the config load/save roundtrip.
- Update the README with clearer language.
- Submit a "good first issue" from the issue tracker.

Don't hesitate to ask questions. We were all beginners once.

---

## Need Help?

Open an issue, start a discussion, or reach out to the maintainers. We're happy to help you get your contribution across the finish line.
