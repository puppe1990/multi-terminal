use crate::layout::{AgentConfig, LayoutMode};

pub fn build_applescript(
    layout_mode: &LayoutMode,
    agents: &[AgentConfig],
    cwd: &str,
) -> Result<String, String> {
    if cwd.is_empty() {
        return Err("current directory empty".to_string());
    }

    // Convert LayoutMode to Layout for now
    let layout = match layout_mode {
        LayoutMode::LegacyA => crate::layout::Layout::A,
        LayoutMode::LegacyB => crate::layout::Layout::B,
        LayoutMode::Dynamic { .. } => crate::layout::Layout::B,
    };

    let panes: Vec<_> = layout
        .panes(agents)
        .into_iter()
        .map(|agent| {
            let cmd = agent.effective_command();
            let title = agent.effective_title();
            (title, cmd)
        })
        .collect();

    let mut lines = vec![
        r#"tell application "Terminal""#.to_string(),
        "  activate".to_string(),
        format!(
            "  do script \"{}\"",
            apple_escape(&pane_command(
                cwd,
                panes[0].1.as_ref().map(|c| c.to_shell_string()).as_deref()
            ))
        ),
    ];

    for (_, cmd) in panes.iter().skip(1) {
        lines.push(format!(
            "  do script \"{}\" in front window",
            apple_escape(&pane_command(
                cwd,
                cmd.as_ref().map(|c| c.to_shell_string()).as_deref()
            ))
        ));
    }

    lines.push(r#"end tell"#.to_string());
    Ok(lines.join("\n"))
}

pub fn run(layout_mode: &LayoutMode, agents: &[AgentConfig]) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;
    let cwd = cwd
        .to_str()
        .ok_or_else(|| "current directory contains invalid characters".to_string())?;

    let script = build_applescript(layout_mode, agents, cwd)?;

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|e| format!("failed to execute osascript: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Terminal.app AppleScript failed with status {status}"
        ))
    }
}

pub fn is_supported() -> bool {
    cfg!(target_os = "macos") && app_exists_in_paths(&terminal_app_candidates())
}

fn terminal_app_candidates() -> Vec<String> {
    vec![
        "/System/Applications/Utilities/Terminal.app".to_string(),
        "/Applications/Utilities/Terminal.app".to_string(),
        "/Applications/Terminal.app".to_string(),
    ]
}

fn cd_command(cwd: &str) -> String {
    format!("cd '{}'", cwd.replace('\'', r"'\''"))
}

fn pane_command(cwd: &str, command: Option<&str>) -> String {
    match command {
        Some(command) => format!("{}; {}", cd_command(cwd), command),
        None => cd_command(cwd),
    }
}

fn apple_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn app_exists_in_paths(paths: &[String]) -> bool {
    paths.iter().any(|path| std::path::Path::new(path).exists())
}
