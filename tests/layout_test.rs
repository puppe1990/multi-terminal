use multi_terminal::layout::{AgentConfig, AgentType, Layout, LayoutMode, LayoutType, SavedLayout};
use multi_terminal::{parse_args, resolve_agents, resolve_runtime_args};

fn default_agents() -> Vec<AgentConfig> {
    Layout::B.default_agents()
}

#[test]
fn layout_b_has_four_panes() {
    let panes = Layout::B.panes(&default_agents());
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_a_has_four_panes() {
    let panes = Layout::A.panes(&default_agents());
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_b_pane0_has_no_command() {
    let panes = Layout::B.panes(&default_agents());
    assert!(panes[0].effective_command().is_none());
}

#[test]
fn layout_b_pane1_runs_claude() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[1].effective_command().unwrap();
    assert_eq!(cmd.program, "claude");
    assert_eq!(cmd.args, vec!["--dangerously-skip-permissions"]);
}

#[test]
fn layout_b_pane2_runs_codex() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[2].effective_command().unwrap();
    assert_eq!(cmd.program, "codex");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_b_pane3_runs_qwen() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[3].effective_command().unwrap();
    assert_eq!(cmd.program, "qwen");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_a_pane0_is_free() {
    let panes = Layout::A.panes(&default_agents());
    assert!(panes[0].effective_command().is_none());
}

#[test]
fn default_layout_is_b() {
    let args = parse_args(&["multi-terminal"]);
    assert_eq!(args.layout, Layout::B);
}

#[test]
fn flag_layout_a_selects_layout_a() {
    let args = parse_args(&["multi-terminal", "--layout", "a"]);
    assert_eq!(args.layout, Layout::A);
}

#[test]
fn flag_layout_b_selects_layout_b() {
    let args = parse_args(&["multi-terminal", "--layout", "b"]);
    assert_eq!(args.layout, Layout::B);
}

#[test]
fn resolve_runtime_args_uses_saved_layout_agents_when_loading() {
    let args = parse_args(&["multi-terminal", "--load", "team"]);
    let saved = SavedLayout {
        layout: "a".to_string(),
        agents: vec![
            AgentConfig::new(AgentType::Custom("htop".to_string())).with_title("Monitor"),
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Custom("npm run dev".to_string())).with_title("App"),
            AgentConfig::new(AgentType::Qwen),
        ],
        maximize: true,
    };

    let resolved = resolve_runtime_args(&args, Some(saved)).unwrap();

    assert_eq!(resolved.layout_mode, LayoutMode::LegacyA);
    assert!(resolved.maximize);
    assert_eq!(resolved.agents[0].effective_title(), "Monitor");
    assert_eq!(
        resolved.agents[2].effective_command().unwrap().program,
        "npm run dev"
    );
}

#[test]
fn resolve_runtime_args_allows_cli_overrides_on_loaded_agents() {
    let args = parse_args(&[
        "multi-terminal",
        "--load",
        "team",
        "--pane2",
        "lazygit",
        "--title2",
        "Git",
        "--no-qwen",
    ]);
    let saved = SavedLayout {
        layout: "b".to_string(),
        agents: Layout::B.default_agents(),
        maximize: false,
    };

    let resolved = resolve_runtime_args(&args, Some(saved)).unwrap();

    assert_eq!(
        resolved.agents[1].effective_command().unwrap().program,
        "lazygit"
    );
    assert_eq!(resolved.agents[1].effective_title(), "Git");
    assert!(resolved.agents[3].effective_command().is_none());
}

#[test]
fn resolve_runtime_args_rejects_loaded_layout_with_wrong_pane_count() {
    let args = parse_args(&["multi-terminal", "--load", "broken"]);
    let saved = SavedLayout {
        layout: "b".to_string(),
        agents: vec![AgentConfig::new(AgentType::Shell)],
        maximize: false,
    };

    let error = resolve_runtime_args(&args, Some(saved)).unwrap_err();

    assert!(error.contains("expected 4 panes"));
}

#[test]
fn resolve_agents_applies_cli_overrides_without_loaded_layout() {
    let args = parse_args(&[
        "multi-terminal",
        "--pane3",
        "npm run dev",
        "--title3",
        "App",
    ]);
    let agents = resolve_agents(&args, None).unwrap();

    assert_eq!(
        agents[2].effective_command().unwrap().program,
        "npm run dev"
    );
    assert_eq!(agents[2].effective_title(), "App");
}

#[test]
fn parse_args_supports_layout_type_and_panes() {
    let args = parse_args(&["multi-terminal", "--layout-type", "grid", "--pane-count", "6"]);

    assert_eq!(args.layout_type, Some(LayoutType::Grid));
    assert_eq!(args.pane_count, Some(6));
}

#[test]
fn resolve_runtime_args_builds_dynamic_defaults() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "main-left",
        "--pane-count",
        "5",
    ]);

    let resolved = resolve_runtime_args(&args, None).unwrap();

    assert_eq!(
        resolved.layout_mode,
        LayoutMode::Dynamic {
            layout_type: LayoutType::MainLeft,
            pane_count: 5,
        }
    );
    assert_eq!(resolved.agents.len(), 5);
    assert!(resolved.agents[0].effective_command().is_none());
    assert_eq!(
        resolved.agents[1].effective_command().unwrap().program,
        "claude"
    );
    assert_eq!(resolved.agents[4].effective_command().is_none(), true);
}

#[test]
fn resolve_agents_applies_indexed_pane_override() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "grid",
        "--pane-count",
        "6",
        "--pane",
        "5=htop",
        "--title",
        "5=Monitor",
    ]);

    let resolved = resolve_runtime_args(&args, None).unwrap();

    assert_eq!(
        resolved.agents[4].effective_command().unwrap().program,
        "htop"
    );
    assert_eq!(resolved.agents[4].effective_title(), "Monitor");
}

#[test]
fn resolve_runtime_args_rejects_override_out_of_bounds() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "grid",
        "--pane-count",
        "3",
        "--pane",
        "4=lazygit",
    ]);

    let error = resolve_runtime_args(&args, None).unwrap_err();

    assert!(error.contains("pane index 4 is out of bounds for 3 panes"));
}
