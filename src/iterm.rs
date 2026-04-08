use crate::layout::Layout;
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

    let panes = build_tab_specs(layout);
    let mut lines = vec![
        r#"tell application "iTerm""#.to_string(),
        "  activate".to_string(),
        "  create window with default profile".to_string(),
        "  tell current window".to_string(),
        "    set pane0 to current session".to_string(),
    ];

    match layout {
        Layout::B => {
            lines.push("    set pane1 to (split horizontally with default profile)".to_string());
            lines.push("    tell pane0".to_string());
            lines.push("      set pane2 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane1".to_string());
            lines.push("      set pane3 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
        }
        Layout::A => {
            lines.push("    set pane1 to (split horizontally with default profile)".to_string());
            lines.push("    tell pane1".to_string());
            lines.push("      set pane2 to (split vertically with default profile)".to_string());
            lines.push("    end tell".to_string());
            lines.push("    tell pane2".to_string());
            lines.push("      set pane3 to (split horizontally with default profile)".to_string());
            lines.push("    end tell".to_string());
        }
    }

    for (index, pane) in panes.iter().enumerate() {
        lines.push(format!("    tell pane{}", index));
        lines.push(format!(
            "      write text \"{}\"",
            apple_escape(&cd_command(cwd))
        ));
        if let Some(command) = &pane.command {
            lines.push(format!(
                "      write text \"{}\"",
                apple_escape(command)
            ));
        }
        lines.push("    end tell".to_string());
    }

    lines.push("  end tell".to_string());
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

pub fn ensure_installed() -> Result<(), String> {
    if !cfg!(target_os = "macos") || is_supported() {
        return Ok(());
    }

    let brew = which::which("brew")
        .map_err(|_| "iTerm2 nao encontrado e Homebrew nao esta instalado".to_string())?;

    let install = build_brew_install_command(
        brew.to_str()
            .ok_or_else(|| "caminho do brew contem caracteres invalidos".to_string())?,
    );

    let status = std::process::Command::new(&install.program)
        .args(&install.args)
        .status()
        .map_err(|e| format!("falha ao executar instalacao do iTerm2 via brew: {e}"))?;

    if !status.success() {
        return Err(format!(
            "instalacao automatica do iTerm2 falhou com status {status}"
        ));
    }

    if is_supported() {
        Ok(())
    } else {
        Err("iTerm2 foi instalado, mas ainda nao ficou detectavel pelo sistema".to_string())
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
