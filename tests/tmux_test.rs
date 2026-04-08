use multi_terminal::layout::{AgentConfig, Layout, LayoutMode, LayoutType};
use multi_terminal::tmux::build_commands;

fn default_agents() -> Vec<AgentConfig> {
    Layout::B.default_agents()
}

#[test]
fn layout_b_commands_start_with_new_session() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_b_sends_claude_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_claude = cmds
        .iter()
        .any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude, "esperado comando claude nos cmds: {:?}", cmds);
}

#[test]
fn layout_b_sends_codex_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_codex = cmds.iter().any(|c| c.contains("codex --yolo"));
    assert!(has_codex);
}

#[test]
fn layout_b_sends_qwen_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_qwen = cmds.iter().any(|c| c.contains("qwen --yolo"));
    assert!(has_qwen);
}

#[test]
fn layout_b_ends_with_attach() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    assert!(cmds.last().unwrap().contains("attach-session"));
}

#[test]
fn layout_a_commands_start_with_new_session() {
    let cmds = build_commands(&LayoutMode::LegacyA, &default_agents(), "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_a_sends_claude_command() {
    let cmds = build_commands(&LayoutMode::LegacyA, &default_agents(), "multi");
    let has_claude = cmds
        .iter()
        .any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude);
}

#[test]
fn build_commands_supports_dynamic_grid_layout() {
    let layout = LayoutMode::Dynamic {
        layout_type: LayoutType::Grid,
        pane_count: 6,
    };
    let agents = layout.default_agents();

    let cmds = build_commands(&layout, &agents, "test-session");

    assert!(cmds.iter().any(|cmd| cmd.contains("split-window")));
    assert_eq!(
        cmds.iter().filter(|cmd| cmd.contains("send-keys")).count(),
        4
    );
    assert!(cmds.iter().any(|cmd| cmd.contains("opencode")));
    assert!(!cmds.iter().any(|cmd| cmd.contains("opencode --yolo")));
}

#[test]
fn build_commands_supports_single_pane_layout() {
    let layout = LayoutMode::Dynamic {
        layout_type: LayoutType::Grid,
        pane_count: 1,
    };
    let agents = layout.default_agents();

    let cmds = build_commands(&layout, &agents, "solo");

    assert!(!cmds.iter().any(|cmd| cmd.contains("split-window")));
}

#[test]
fn build_commands_main_left_uses_subgrid_splits() {
    let layout = LayoutMode::Dynamic {
        layout_type: LayoutType::MainLeft,
        pane_count: 5,
    };

    let cmds = build_commands(&layout, &layout.default_agents(), "subgrid");
    let horizontal_splits = cmds
        .iter()
        .filter(|cmd| cmd.contains("split-window -h"))
        .count();
    let vertical_splits = cmds
        .iter()
        .filter(|cmd| cmd.contains("split-window -v"))
        .count();

    assert_eq!(horizontal_splits, 2);
    assert_eq!(vertical_splits, 2);
}

#[test]
fn build_commands_main_top_uses_subgrid_splits() {
    let layout = LayoutMode::Dynamic {
        layout_type: LayoutType::MainTop,
        pane_count: 5,
    };

    let cmds = build_commands(&layout, &layout.default_agents(), "subgrid");
    let horizontal_splits = cmds
        .iter()
        .filter(|cmd| cmd.contains("split-window -h"))
        .count();
    let vertical_splits = cmds
        .iter()
        .filter(|cmd| cmd.contains("split-window -v"))
        .count();

    assert_eq!(horizontal_splits, 1);
    assert_eq!(vertical_splits, 3);
}
