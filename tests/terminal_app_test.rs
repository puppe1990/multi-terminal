use multi_terminal::layout::{AgentConfig, Layout};
use multi_terminal::terminal_app::build_applescript;

fn default_agents() -> Vec<AgentConfig> {
    Layout::B.default_agents()
}

#[test]
fn terminal_app_script_uses_working_directory_for_every_tab() {
    let script = build_applescript(&Layout::B, &default_agents(), "/tmp/my-project").unwrap();

    assert!(script.contains("cd '/tmp/my-project'"));
    assert!(script.matches("cd '/tmp/my-project'").count() >= 4);
}

#[test]
fn terminal_app_script_runs_agent_commands_in_tabs() {
    let script = build_applescript(&Layout::B, &default_agents(), "/tmp/my-project").unwrap();

    assert!(script.contains("do script \"cd '/tmp/my-project'\""));
    assert!(script.contains(
        "do script \"cd '/tmp/my-project'; claude --dangerously-skip-permissions\" in front window"
    ));
    assert!(script.contains("do script \"cd '/tmp/my-project'; codex --yolo\" in front window"));
    assert!(script.contains("do script \"cd '/tmp/my-project'; qwen --yolo\" in front window"));
    assert!(!script.contains("selected tab of front window"));
}
