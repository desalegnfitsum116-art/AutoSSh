use std::fs;
use std::path::PathBuf;

const DESKTOP_FILE: &str = r#"[Desktop Entry]
Type=Application
Name=AutoSSH
Exec={}
Comment=Automatic SSH connection manager
Terminal=false
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#;

fn get_binary_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

fn get_autostart_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push("autostart");
        p.push("auto-ssh.desktop");
        p
    })
}

pub fn enable() -> Result<(), String> {
    let exe_path = get_binary_path().ok_or("Cannot determine executable path")?;
    let autostart = get_autostart_path().ok_or("Cannot determine autostart directory")?;
    let parent = autostart.parent().ok_or("Invalid autostart path")?;

    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create autostart directory: {}", e))?;

    let contents = DESKTOP_FILE.replace("{}", &exe_path);
    fs::write(&autostart, &contents)
        .map_err(|e| format!("Failed to write autostart file: {}", e))?;

    log::info!("Autostart enabled at {}", autostart.display());
    Ok(())
}

pub fn disable() -> Result<(), String> {
    if let Some(path) = get_autostart_path() {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
            log::info!("Autostart disabled");
        }
    }
    Ok(())
}
