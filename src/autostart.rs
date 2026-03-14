// autostart.rs — Auto-launch at macOS login via LaunchAgent
//
// Creates or removes a LaunchAgent plist file to control whether
// mactemp starts automatically when the user logs in. This is the
// standard macOS mechanism for user-level login items.
//
// Plist location: ~/Library/LaunchAgents/com.mactemp.monitor.plist

use std::env;
use std::fs;
use std::path::PathBuf;

/// The LaunchAgent bundle identifier.
const LAUNCH_AGENT_LABEL: &str = "com.mactemp.monitor";

/// Returns the path to the LaunchAgent plist file.
/// Location: ~/Library/LaunchAgents/com.mactemp.monitor.plist
fn plist_path() -> Option<PathBuf> {
    dirs_replacement().map(|home| {
        home.join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", LAUNCH_AGENT_LABEL))
    })
}

/// Gets the user's home directory without the `dirs` crate.
/// Uses the HOME environment variable which is always set on macOS.
fn dirs_replacement() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}

/// Checks whether auto-launch at login is currently enabled.
/// Simply checks for the existence of the LaunchAgent plist file.
pub fn is_enabled() -> bool {
    plist_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// Toggles the auto-launch state.
/// If currently enabled, disables it by removing the plist.
/// If currently disabled, enables it by writing the plist.
///
/// Returns the new state: `true` if now enabled, `false` if now disabled.
pub fn toggle() -> bool {
    if is_enabled() {
        disable();
        false
    } else {
        enable();
        true
    }
}

/// Enables auto-launch by writing the LaunchAgent plist file.
fn enable() {
    let binary_path = match env::current_exe() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            eprintln!("[mactemp] Failed to get binary path: {:?}", err);
            return;
        }
    };

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>"#,
        label = LAUNCH_AGENT_LABEL,
        binary = binary_path,
    );

    if let Some(path) = plist_path() {
        // Ensure the LaunchAgents directory exists
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("[mactemp] Failed to create LaunchAgents dir: {:?}", err);
                return;
            }
        }

        match fs::write(&path, plist_content) {
            Ok(_) => eprintln!("[mactemp] Auto-launch enabled: {:?}", path),
            Err(err) => eprintln!("[mactemp] Failed to write plist: {:?}", err),
        }
    }
}

/// Disables auto-launch by removing the LaunchAgent plist file.
fn disable() {
    if let Some(path) = plist_path() {
        match fs::remove_file(&path) {
            Ok(_) => eprintln!("[mactemp] Auto-launch disabled"),
            Err(err) => eprintln!("[mactemp] Failed to remove plist: {:?}", err),
        }
    }
}
