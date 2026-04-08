use multi_terminal::layout::Layout;
use multi_terminal::tmux::build_commands;

#[test]
fn all_four_panes_are_configured_in_layout_b() {
    let cmds = build_commands(&Layout::B, "test-session");
    // Deve ter: new-session, 4 splits, 3 send-keys, select-pane, attach
    assert!(cmds.len() >= 9);
}

#[test]
fn all_four_panes_are_configured_in_layout_a() {
    let cmds = build_commands(&Layout::A, "test-session");
    assert!(cmds.len() >= 9);
}

#[test]
fn session_name_appears_in_all_commands() {
    let session = "my-custom-session";
    let cmds = build_commands(&Layout::B, session);
    for cmd in &cmds {
        assert!(cmd.contains(session), "comando sem session name: {}", cmd);
    }
}
