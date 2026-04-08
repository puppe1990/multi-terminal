use multi_terminal::layout::{AgentConfig, LayoutMode};
use multi_terminal::tmux::build_commands;
use std::fs;

fn default_agents() -> Vec<AgentConfig> {
    LayoutMode::LegacyB.default_agents()
}

#[test]
fn all_four_panes_are_configured_in_layout_b() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "test-session");
    // Deve ter: new-session, 4 splits, 3 send-keys, select-pane, attach
    assert!(cmds.len() >= 9);
}

#[test]
fn all_four_panes_are_configured_in_layout_a() {
    let cmds = build_commands(&LayoutMode::LegacyA, &default_agents(), "test-session");
    assert!(cmds.len() >= 9);
}

#[test]
fn session_name_appears_in_all_commands() {
    let session = "my-custom-session";
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), session);
    for cmd in &cmds {
        assert!(cmd.contains(session), "comando sem session name: {}", cmd);
    }
}

#[test]
fn install_script_forces_reinstall_of_global_binary() {
    let script = fs::read_to_string("install").expect("deve ler script install");
    assert!(
        script.contains("cargo install --path . --force"),
        "script install deve forcar reinstalacao do binario global"
    );
}
