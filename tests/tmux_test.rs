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
fn layout_b_sends_opencode_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_opencode = cmds.iter().any(|c| c.contains("opencode"));
    assert!(has_opencode);
}

#[test]
fn layout_b_sends_grok_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_grok = cmds.iter().any(|c| c.contains("grok --yolo"));
    assert!(has_grok, "esperado comando grok nos cmds: {:?}", cmds);
}

#[test]
fn layout_b_sends_agent_command() {
    let cmds = build_commands(&LayoutMode::LegacyB, &default_agents(), "multi");
    let has_agent = cmds.iter().any(|c| c.contains("'agent'"));
    assert!(has_agent);
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
fn layout_a_sends_opencode_command() {
    let cmds = build_commands(&LayoutMode::LegacyA, &default_agents(), "multi");
    let has_opencode = cmds.iter().any(|c| c.contains("opencode"));
    assert!(has_opencode);
}

#[test]
fn build_commands_supports_dynamic_grid_layout() {
    let layout = LayoutMode::Dynamic {
        layout_type: LayoutType::Grid,
        pane_count: 6,
    };
    let agents = layout.default_agents();

    // Verify global panes for --layout-type grid --panes 6 also place the cursor/agent on pane 5 (index 4)
    assert!(agents[0].effective_command().is_none());
    assert_eq!(agents[1].effective_command().unwrap().program, "opencode");
    assert_eq!(agents[2].effective_command().unwrap().program, "grok");
    assert!(agents[3].effective_command().is_none());
    assert_eq!(agents[4].effective_command().unwrap().program, "agent");
    assert_eq!(agents[5].effective_command().unwrap().program, "kilo");

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
