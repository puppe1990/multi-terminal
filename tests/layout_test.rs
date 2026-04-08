use multi_terminal::layout::Layout;

#[test]
fn layout_b_has_four_panes() {
    let panes = Layout::B.panes();
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_a_has_four_panes() {
    let panes = Layout::A.panes();
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_b_pane0_has_no_command() {
    let panes = Layout::B.panes();
    assert!(panes[0].command.is_none());
}

#[test]
fn layout_b_pane1_runs_claude() {
    let panes = Layout::B.panes();
    let cmd = panes[1].command.as_ref().unwrap();
    assert_eq!(cmd.program, "claude");
    assert_eq!(cmd.args, vec!["--dangerously-skip-permissions"]);
}

#[test]
fn layout_b_pane2_runs_codex() {
    let panes = Layout::B.panes();
    let cmd = panes[2].command.as_ref().unwrap();
    assert_eq!(cmd.program, "codex");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_b_pane3_runs_qwen() {
    let panes = Layout::B.panes();
    let cmd = panes[3].command.as_ref().unwrap();
    assert_eq!(cmd.program, "qwen");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_a_pane0_is_free() {
    let panes = Layout::A.panes();
    assert!(panes[0].command.is_none());
}
