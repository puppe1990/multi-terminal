use crate::layout::{AgentConfig, LayoutMode, SplitDirection};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct TabSpec {
    pub title: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub fn build_tab_specs(layout_mode: &LayoutMode, agents: &[AgentConfig]) -> Vec<TabSpec> {
    let _ = layout_mode;

    agents
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, agent)| {
            let cmd = agent.effective_command();
            TabSpec {
                title: pane_title(index, &agent),
                command: cmd.map(|c| c.to_shell_string()),
            }
        })
        .collect()
}

pub fn build_applescript(
    layout_mode: &LayoutMode,
    agents: &[AgentConfig],
    maximize: bool,
    cwd: &str,
) -> Result<String, String> {
    if cwd.is_empty() {
        return Err("current directory empty".to_string());
    }

    let panes = build_tab_specs(layout_mode, agents);
    let mut lines = vec![
        r#"use framework "AppKit""#.to_string(),
        "use scripting additions".to_string(),
        "".to_string(),
        "on screenBoundsForBounds(windowBounds)".to_string(),
        "  set mainFrame to ((current application's NSScreen's mainScreen())'s frame())"
            .to_string(),
        "  set mainHeight to item 2 of item 2 of mainFrame".to_string(),
        "  if windowBounds is missing value then".to_string(),
        "    return missing value".to_string(),
        "  end if".to_string(),
        "  set centerX to ((item 1 of windowBounds) + (item 3 of windowBounds)) / 2".to_string(),
        "  set centerY to ((item 2 of windowBounds) + (item 4 of windowBounds)) / 2".to_string(),
        "  set appKitCenterY to mainHeight - centerY".to_string(),
        "  set screenList to current application's NSScreen's screens()".to_string(),
        "  repeat with screenRef in screenList".to_string(),
        "    set screenFrame to (screenRef's frame())".to_string(),
        "    set screenOrigin to item 1 of screenFrame".to_string(),
        "    set screenSize to item 2 of screenFrame".to_string(),
        "    set minX to item 1 of screenOrigin".to_string(),
        "    set minY to item 2 of screenOrigin".to_string(),
        "    set screenWidth to item 1 of screenSize".to_string(),
        "    set screenHeight to item 2 of screenSize".to_string(),
        "    set maxX to minX + screenWidth".to_string(),
        "    set maxY to minY + screenHeight".to_string(),
        "    if centerX >= minX and centerX < maxX and appKitCenterY >= minY and appKitCenterY < maxY then".to_string(),
        "      set topY to mainHeight - maxY".to_string(),
        "      set bottomY to mainHeight - minY".to_string(),
        "      return {minX as integer, topY as integer, maxX as integer, bottomY as integer}"
            .to_string(),
        "    end if".to_string(),
        "  end repeat".to_string(),
        "  return missing value".to_string(),
        "end screenBoundsForBounds".to_string(),
        "".to_string(),
        r#"tell application "Finder""#.to_string(),
        "  set screenBounds to bounds of window of desktop".to_string(),
        "end tell".to_string(),
        "set callerBounds to missing value".to_string(),
        "try".to_string(),
        r#"  tell application "System Events""#.to_string(),
        "    set frontProcess to first application process whose frontmost is true".to_string(),
        "    if exists window 1 of frontProcess then".to_string(),
        "      set {callerX, callerY} to position of window 1 of frontProcess".to_string(),
        "      set {callerWidth, callerHeight} to size of window 1 of frontProcess".to_string(),
        "      set callerBounds to {callerX, callerY, callerX + callerWidth, callerY + callerHeight}"
            .to_string(),
        "    end if".to_string(),
        "  end tell".to_string(),
        "end try".to_string(),
        "set callerScreenBounds to my screenBoundsForBounds(callerBounds)".to_string(),
        "if callerScreenBounds is not missing value then".to_string(),
        "  set screenBounds to callerScreenBounds".to_string(),
        "end if".to_string(),
        r#"tell application "iTerm""#.to_string(),
        "  activate".to_string(),
        "  create window with default profile".to_string(),
        "  tell current window".to_string(),
    ];

    if maximize {
        lines.push("    set bounds to screenBounds".to_string());
    }

    lines.push("    set pane0 to current session".to_string());
    for split in layout_mode.split_operations() {
        let split_command = match split.direction {
            SplitDirection::Horizontal => "split horizontally with default profile",
            SplitDirection::Vertical => "split vertically with default profile",
        };

        lines.push(format!("    tell pane{}", split.parent));
        lines.push(format!(
            "      set pane{} to ({})",
            split.new_index, split_command
        ));
        lines.push("    end tell".to_string());
    }

    // Set custom names for each pane
    for (index, pane) in panes.iter().enumerate() {
        lines.push(format!("    tell pane{}", index));
        lines.push(format!(
            "      set name to \"{}\"",
            apple_escape(&pane.title)
        ));
        lines.push("    end tell".to_string());
    }

    for (index, pane) in panes.iter().enumerate() {
        lines.push(format!("    tell pane{}", index));
        lines.push(format!(
            "      write text \"{}\"",
            apple_escape(&cd_command(cwd))
        ));
        if let Some(command) = &pane.command {
            lines.push(format!("      write text \"{}\"", apple_escape(command)));
        }
        lines.push("    end tell".to_string());
    }

    lines.push("  end tell".to_string());
    lines.push(r#"end tell"#.to_string());
    Ok(lines.join("\n"))
}

pub fn run(layout_mode: &LayoutMode, agents: &[AgentConfig], maximize: bool) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;
    let cwd = cwd
        .to_str()
        .ok_or_else(|| "current directory contains invalid characters".to_string())?;

    let script = build_applescript(layout_mode, agents, maximize, cwd)?;

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|e| format!("failed to execute osascript: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("iTerm2 AppleScript failed with status {status}"))
    }
}

pub fn is_supported() -> bool {
    cfg!(target_os = "macos") && (iterm_app_exists() || app_exists_via_mdfind())
}

pub fn ensure_installed() -> Result<(), String> {
    if !cfg!(target_os = "macos") || is_supported() {
        return Ok(());
    }

    let brew = which::which("brew")
        .map_err(|_| "iTerm2 not found and Homebrew not installed".to_string())?;

    let install = build_brew_install_command(
        brew.to_str()
            .ok_or_else(|| "brew path contains invalid characters".to_string())?,
    );

    let status = std::process::Command::new(&install.program)
        .args(&install.args)
        .status()
        .map_err(|e| format!("failed to install iTerm2 via brew: {e}"))?;

    if !status.success() {
        return Err(format!(
            "automatic iTerm2 installation failed with status {status}"
        ));
    }

    if is_supported() {
        Ok(())
    } else {
        Err("iTerm2 was installed but still not detectable".to_string())
    }
}

pub fn build_brew_install_command(brew_path: &str) -> ShellCommand {
    ShellCommand {
        program: brew_path.to_string(),
        args: vec![
            "install".to_string(),
            "--cask".to_string(),
            "iterm2".to_string(),
        ],
    }
}

fn iterm_app_exists() -> bool {
    let candidates = vec![
        "/Applications/iTerm.app".to_string(),
        "/Applications/iTerm2.app".to_string(),
        "/System/Applications/iTerm.app".to_string(),
        format!(
            "{}/Applications/iTerm.app",
            std::env::var("HOME").unwrap_or_default()
        ),
        format!(
            "{}/Applications/iTerm2.app",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    app_exists_in_paths(&candidates)
}

pub fn app_exists_in_paths(paths: &[impl AsRef<str>]) -> bool {
    paths.iter().any(|path| Path::new(path.as_ref()).exists())
}

fn app_exists_via_mdfind() -> bool {
    std::process::Command::new("mdfind")
        .arg("kMDItemCFBundleIdentifier == 'com.googlecode.iterm2'")
        .output()
        .map(|output| !String::from_utf8_lossy(&output.stdout).trim().is_empty())
        .unwrap_or(false)
}

fn pane_title(_index: usize, agent: &AgentConfig) -> String {
    agent.effective_title()
}

fn cd_command(cwd: &str) -> String {
    format!("cd '{}'", cwd.replace('\'', r"'\''"))
}

fn apple_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
