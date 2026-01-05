use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxPane {
    pub session_name: String,
    pub window_index: u32,
    pub window_name: String,
    pub pane_index: u32,
    pub pane_id: String,
    pub is_active: bool,
}

fn run_tmux_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute tmux: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("tmux command failed: {}", stderr.trim()))
    }
}

pub fn is_tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_panes() -> Result<Vec<TmuxPane>, String> {
    let format = "#{session_name}|#{window_index}|#{window_name}|#{pane_index}|#{pane_id}|#{pane_active}";
    let output = run_tmux_command(&["list-panes", "-a", "-F", format])?;

    let panes: Vec<TmuxPane> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                Some(TmuxPane {
                    session_name: parts[0].to_string(),
                    window_index: parts[1].parse().unwrap_or(0),
                    window_name: parts[2].to_string(),
                    pane_index: parts[3].parse().unwrap_or(0),
                    pane_id: parts[4].to_string(),
                    is_active: parts[5] == "1",
                })
            } else {
                None
            }
        })
        .collect();

    Ok(panes)
}

pub fn capture_pane(pane_id: &str) -> Result<String, String> {
    // -S -: start from the beginning of history
    // -E -: end at the last line
    run_tmux_command(&["capture-pane", "-p", "-S", "-", "-E", "-", "-t", pane_id])
}

pub fn send_keys(pane_id: &str, keys: &str) -> Result<(), String> {
    log::info!(target: "eocc.tmux", "send_keys: pane_id={}, keys={}", pane_id, keys);
    let result = run_tmux_command(&["send-keys", "-t", pane_id, keys]);
    log::info!(target: "eocc.tmux", "send_keys result: {:?}", result);
    result?;
    Ok(())
}
