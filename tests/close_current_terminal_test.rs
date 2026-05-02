use multi_terminal::current_terminal::{
    build_close_window_script, build_force_close_window_script, parse_term_program, CloseRequest,
    TerminalApp,
};

#[test]
fn parse_term_program_accepts_supported_terminal_apps() {
    assert_eq!(
        parse_term_program(Some("Apple_Terminal")).unwrap(),
        Some(TerminalApp::Terminal)
    );
    assert_eq!(
        parse_term_program(Some("iTerm.app")).unwrap(),
        Some(TerminalApp::ITerm)
    );
}

#[test]
fn close_window_script_targets_captured_terminal_window() {
    let terminal_script = build_close_window_script(&CloseRequest {
        app: TerminalApp::Terminal,
        window_id: 42,
    });
    let iterm_script = build_close_window_script(&CloseRequest {
        app: TerminalApp::ITerm,
        window_id: 7,
    });

    assert!(terminal_script.contains("tell application \"Terminal\""));
    assert!(terminal_script.contains("first window whose id is 42"));
    assert!(iterm_script.contains("tell application \"iTerm\""));
    assert!(iterm_script.contains("first window whose id is 7"));
}

#[test]
fn force_close_script_quits_terminal_session_without_prompt() {
    let terminal_script = build_force_close_window_script(&CloseRequest {
        app: TerminalApp::Terminal,
        window_id: 42,
    });
    let iterm_script = build_force_close_window_script(&CloseRequest {
        app: TerminalApp::ITerm,
        window_id: 7,
    });

    assert!(terminal_script.contains("do script \"exit\""));
    assert!(terminal_script.contains("in first tab of first window whose id is 42"));
    assert!(terminal_script.contains("close (first window whose id is 42) saving no"));
    assert!(iterm_script.contains("tell current session of first window whose id is 7"));
    assert!(iterm_script.contains("write text \"exit\""));
    assert!(iterm_script.contains("close (first window whose id is 7)"));
}
