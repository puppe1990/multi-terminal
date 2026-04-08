use multi_terminal::layout::Layout;
use multi_terminal::tmux::build_commands;

#[test]
fn layout_b_commands_start_with_new_session() {
    let cmds = build_commands(&Layout::B, "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_b_sends_claude_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_claude = cmds.iter().any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude, "esperado comando claude nos cmds: {:?}", cmds);
}

#[test]
fn layout_b_sends_codex_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_codex = cmds.iter().any(|c| c.contains("codex --yolo"));
    assert!(has_codex);
}

#[test]
fn layout_b_sends_qwen_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_qwen = cmds.iter().any(|c| c.contains("qwen --yolo"));
    assert!(has_qwen);
}

#[test]
fn layout_b_ends_with_attach() {
    let cmds = build_commands(&Layout::B, "multi");
    assert!(cmds.last().unwrap().contains("attach-session"));
}

#[test]
fn layout_a_commands_start_with_new_session() {
    let cmds = build_commands(&Layout::A, "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_a_sends_claude_command() {
    let cmds = build_commands(&Layout::A, "multi");
    let has_claude = cmds.iter().any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude);
}
