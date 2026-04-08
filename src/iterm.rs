use crate::layout::{AgentConfig, Layout};
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

pub fn build_tab_specs(layout: &Layout, agents: &[AgentConfig]) -> Vec<TabSpec> {
    layout
        .panes(agents)
        .into_iter()
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
    layout: &Layout,
    agents: &[AgentConfig],
    maximize: bool,
    cwd: &str,
) -> Result<String, String> {
    if cwd.is_empty() {
        return Err("current directory empty".to_string());
    }

    let panes = build_tab_specs(layout, agents);
    let mut lines = vec![
        r#"tell application "Finder""#.to_string(),
        "  set screenBounds to bounds of window of desktop".to_string(),
        "end tell".to_string(),
        r#"tell application "iTerm""#.to_string(),
        "  activate".to_string(),
        "  create window with default profile".to_string(),
        "  tell current window".to_string(),
    ];

    if maximize {
        lines.push("    set bounds to screenBounds".to_string());
    }

    lines.push("    set pane0 to current session".to_string());

    match layout {
        Layout::B => {
            lines.push("    tell pane0".to_string());
            lines.push("      set pane1 to (split horizontally with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane0".to_string());
            lines.push("      set pane2 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane1".to_string());
            lines.push("      set pane3 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
        }
        Layout::A => {
            lines.push("    tell pane0".to_string());
            lines.push("      set pane1 to (split horizontally with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane1".to_string());
            lines.push("      set pane2 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane2".to_string());
            lines.push("      set pane3 to (split horizontally with default profile)".to_string());
            lines.push("    end tell".to_string());
        }
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

pub fn run(layout: &Layout, agents: &[AgentConfig], maximize: bool) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;
    let cwd = cwd
        .to_str()
        .ok_or_else(|| "current directory contains invalid characters".to_string())?;

    let script = build_applescript(layout, agents, maximize, cwd)?;

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
