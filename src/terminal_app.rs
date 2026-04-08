use crate::iterm::build_tab_specs;
use crate::layout::Layout;

pub fn build_applescript(layout: &Layout, cwd: &str) -> Result<String, String> {
    if cwd.is_empty() {
        return Err("diretorio atual vazio".to_string());
    }

    let tabs = build_tab_specs(layout);
    let mut lines = vec![
        r#"tell application "Terminal""#.to_string(),
        "  activate".to_string(),
        format!(
            "  do script \"{}\"",
            apple_escape(&cd_command(cwd))
        ),
    ];

    for tab in tabs.iter().skip(1) {
        lines.push(format!(
            "  do script \"{}\" in front window",
            apple_escape(&cd_command(cwd))
        ));

        if let Some(command) = &tab.command {
            lines.push(format!(
                "  do script \"{}\" in selected tab of front window",
                apple_escape(command)
            ));
        }
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
        Err(format!("AppleScript do Terminal.app falhou com status {status}"))
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

fn apple_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn app_exists_in_paths(paths: &[String]) -> bool {
    paths.iter().any(|path| std::path::Path::new(path).exists())
}
