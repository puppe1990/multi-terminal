use multi_terminal::iterm::{
    app_exists_in_paths, build_applescript, build_brew_install_command, build_tab_specs,
};
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
fn applescript_sends_agent_commands_to_splits() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert!(script.contains("claude --dangerously-skip-permissions"));
    assert!(script.contains("codex --yolo"));
    assert!(script.contains("qwen --yolo"));
    assert!(script.contains("split horizontally with default profile"));
    assert!(script.contains("split vertically with default profile"));
    assert!(!script.contains("create tab with default profile"));
}

#[test]
fn layout_b_applescript_creates_three_splits_for_four_panes() {
    let script = build_applescript(&Layout::B, "/tmp/my-project").unwrap();

    assert_eq!(
        script.matches("split ").count(),
        3,
        "script should create exactly three splits: {script}"
    );
}

#[test]
fn layout_a_applescript_uses_mixed_split_directions() {
    let script = build_applescript(&Layout::A, "/tmp/my-project").unwrap();

    assert!(script.contains("split horizontally with default profile"));
    assert!(script.contains("split vertically with default profile"));
}

#[test]
fn iterm_path_detection_checks_candidate_paths() {
    let missing = ["/definitely/missing/iTerm.app"];
    assert!(!app_exists_in_paths(&missing));
}

#[test]
fn brew_install_command_targets_iterm2_cask() {
    let command = build_brew_install_command("/opt/homebrew/bin/brew");

    assert_eq!(command.program, "/opt/homebrew/bin/brew");
    assert_eq!(command.args, vec!["install", "--cask", "iterm2"]);
}
