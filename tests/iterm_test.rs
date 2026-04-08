use multi_terminal::iterm::{build_applescript, build_tab_specs};
use multi_terminal::layout::Layout;

#[test]
fn build_tab_specs_keeps_first_tab_as_shell() {
    let tabs = build_tab_specs(&Layout::B);

    assert_eq!(tabs.len(), 4);
    assert_eq!(tabs[0].title, "shell");
    assert!(tabs[0].command.is_none());
}

#[test]
fn build_tab_specs_autoruns_agents_in_remaining_tabs() {
    let tabs = build_tab_specs(&Layout::B);

    assert_eq!(
        tabs[1].command.as_deref(),
        Some("claude --dangerously-skip-permissions")
    );
    assert_eq!(tabs[2].command.as_deref(), Some("codex --yolo"));
    assert_eq!(tabs[3].command.as_deref(), Some("qwen --yolo"));
}

#[test]
fn applescript_uses_working_directory_for_every_tab() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert!(script.contains("cd '/tmp/my-project'"));
    assert!(script.matches("cd '/tmp/my-project'").count() >= 4);
}

#[test]
fn applescript_sends_agent_commands_to_tabs() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert!(script.contains("claude --dangerously-skip-permissions"));
    assert!(script.contains("codex --yolo"));
    assert!(script.contains("qwen --yolo"));
}
