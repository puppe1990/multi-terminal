#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalApp {
    ITerm,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseRequest {
    pub app: TerminalApp,
    pub window_id: i64,
}

/// Resolve the terminal app from `TERM_PROGRAM`.
///
/// Example: `parse_term_program(Some("iTerm.app"))`.
pub fn parse_term_program(term_program: Option<&str>) -> Result<Option<TerminalApp>, String> {
    let Some(term_program) = term_program else {
        return Ok(None);
    };

    match term_program {
        "Apple_Terminal" => Ok(Some(TerminalApp::Terminal)),
        "iTerm.app" => Ok(Some(TerminalApp::ITerm)),
        other => Err(format!(
            "unsupported TERM_PROGRAM '{other}', expected 'Apple_Terminal' or 'iTerm.app'"
        )),
    }
}

/// Build the AppleScript that closes the captured terminal window.
///
/// Example: `build_close_window_script(&request)`.
pub fn build_close_window_script(request: &CloseRequest) -> String {
    let app_name = match request.app {
        TerminalApp::ITerm => "iTerm",
        TerminalApp::Terminal => "Terminal",
    };

    format!(
        "tell application \"{app_name}\"\n  if (count of windows) > 0 then\n    close (first window whose id is {})\n  end if\nend tell",
        request.window_id
    )
}

/// Build the AppleScript that force closes the captured terminal window.
///
/// Example: `build_force_close_window_script(&request)`.
pub fn build_force_close_window_script(request: &CloseRequest) -> String {
    match request.app {
        TerminalApp::Terminal => format!(
            "tell application \"Terminal\"\n  if (count of windows) > 0 then\n    do script \"exit\" in first tab of first window whose id is {}\n    delay 0.2\n    close (first window whose id is {}) saving no\n  end if\nend tell",
            request.window_id, request.window_id
        ),
        TerminalApp::ITerm => format!(
            "tell application \"iTerm\"\n  if (count of windows) > 0 then\n    tell current session of first window whose id is {}\n      write text \"exit\"\n    end tell\n    delay 0.2\n    close (first window whose id is {})\n  end if\nend tell",
            request.window_id, request.window_id
        ),
    }
}

fn build_capture_window_script(app: TerminalApp) -> &'static str {
    match app {
        TerminalApp::ITerm => "tell application \"iTerm\"\n  if (count of windows) is 0 then return \"\"\n  return id of current window\nend tell",
        TerminalApp::Terminal => "tell application \"Terminal\"\n  if (count of windows) is 0 then return \"\"\n  return id of front window\nend tell",
    }
}

fn parse_window_id(raw_id: &str) -> Result<i64, String> {
    raw_id
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("invalid terminal window id '{raw_id}', expected integer"))
}

fn run_osascript(script: &str) -> Result<String, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("failed to execute osascript: {e}"))?;

    if !output.status.success() {
        return Err(format!("osascript failed with status {}", output.status));
    }

    String::from_utf8(output.stdout)
        .map(|stdout| stdout.trim().to_string())
        .map_err(|e| format!("invalid osascript output: {e}"))
}

pub fn capture_close_request(enabled: bool) -> Result<Option<CloseRequest>, String> {
    if !enabled || !cfg!(target_os = "macos") {
        return Ok(None);
    }

    let app = match parse_term_program(std::env::var("TERM_PROGRAM").ok().as_deref())? {
        Some(app) => app,
        None => return Ok(None),
    };
    let raw_id = run_osascript(build_capture_window_script(app))?;

    if raw_id.is_empty() {
        return Ok(None);
    }

    Ok(Some(CloseRequest {
        app,
        window_id: parse_window_id(&raw_id)?,
    }))
}

/// Close the previously captured terminal window when available.
///
/// Example: `close_if_requested(close_request.as_ref(), false)`.
pub fn close_if_requested(request: Option<&CloseRequest>, force: bool) {
    let Some(request) = request else {
        return;
    };

    let script = if force {
        build_force_close_window_script(request)
    } else {
        build_close_window_script(request)
    };

    if let Err(error) = run_osascript(&script) {
        eprintln!("Warning: failed to close current terminal: {error}");
    }
}
