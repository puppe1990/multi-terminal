use multi_terminal::layout::Layout;
use multi_terminal::terminal_app::build_applescript;

#[test]
fn terminal_app_script_uses_working_directory_for_every_tab() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert!(script.contains("cd '/tmp/my-project'"));
    assert!(script.matches("cd '/tmp/my-project'").count() >= 4);
}

#[test]
fn terminal_app_script_runs_agent_commands_in_tabs() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert!(script.contains("do script \"cd '/tmp/my-project'\""));
    assert!(script.contains("claude --dangerously-skip-permissions"));
    assert!(script.contains("codex --yolo"));
    assert!(script.contains("qwen --yolo"));
}
