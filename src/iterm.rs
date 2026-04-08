use crate::layout::Layout;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct TabSpec {
    pub title: String,
    pub command: Option<String>,
}

pub fn build_tab_specs(layout: &Layout) -> Vec<TabSpec> {
    layout
        .panes()
        .into_iter()
        .enumerate()
        .map(|(index, pane)| TabSpec {
            title: pane_title(index, pane.command.as_ref().map(|cmd| cmd.to_shell_string())),
            command: pane.command.map(|cmd| cmd.to_shell_string()),
        })
        .collect()
}

pub fn build_applescript(layout: &Layout, cwd: &str) -> Result<String, String> {
    if cwd.is_empty() {
        return Err("diretorio atual vazio".to_string());
    }

    let tabs = build_tab_specs(layout);
    let mut lines = vec![
        r#"tell application "iTerm2""#.to_string(),
        "  activate".to_string(),
        "  create window with default profile".to_string(),
    ];

    for (index, tab) in tabs.iter().enumerate() {
        if index == 0 {
            lines.push("  tell current session of current window".to_string());
        } else {
            lines.push("  tell current window".to_string());
            lines.push("    create tab with default profile".to_string());
            lines.push("  end tell".to_string());
            lines.push("  tell current session of current window".to_string());
        }

        lines.push(format!(
            "    write text \"{}\"",
            apple_escape(&cd_command(cwd))
        ));

        if let Some(command) = &tab.command {
            lines.push(format!(
                "    write text \"{}\"",
                apple_escape(command)
            ));
        }

        lines.push("  end tell".to_string());
    }

    lines.push(r#"end tell"#.to_string());
    Ok(lines.join("\n"))
}

pub fn run(layout: &Layout) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("falha ao obter diretorio atual: {e}"))?;
    let cwd = cwd
        .to_str()
        .ok_or_else(|| "diretorio atual contem caracteres invalidos".to_string())?;

    let script = build_applescript(layout, cwd)?;

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|e| format!("falha ao executar osascript: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("AppleScript do iTerm2 falhou com status {status}"))
    }
}

pub fn is_supported() -> bool {
    cfg!(target_os = "macos") && (iterm_app_exists() || app_exists_via_mdfind())
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

fn pane_title(index: usize, command: Option<String>) -> String {
    if index == 0 {
        "shell".to_string()
    } else {
        match command {
            Some(command) => command
                .split_whitespace()
                .next()
                .unwrap_or("shell")
                .to_string(),
            None => "shell".to_string(),
        }
    }
}

fn cd_command(cwd: &str) -> String {
    format!("cd '{}'", cwd.replace('\'', r"'\''"))
}

fn apple_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
