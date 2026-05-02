use clap::Parser;
use multi_terminal::layout::{AgentConfig, AgentType, Layout, LayoutMode, LayoutType, SavedLayout};
use multi_terminal::{parse_args, resolve_agents, resolve_runtime_args, resolve_working_dir, Args};
use std::path::Path;

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
fn layout_b_pane1_runs_codex() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[1].effective_command().unwrap();
    assert_eq!(cmd.program, "codex");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_b_pane2_runs_kimi() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[2].effective_command().unwrap();
    assert_eq!(cmd.program, "kimi");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_b_pane3_runs_opencode() {
    let panes = Layout::B.panes(&default_agents());
    let cmd = panes[3].effective_command().unwrap();
    assert_eq!(cmd.program, "opencode");
    assert!(cmd.args.is_empty());
}

#[test]
fn layout_a_pane0_is_free() {
    let panes = Layout::A.panes(&default_agents());
    assert!(panes[0].effective_command().is_none());
}

#[test]
fn default_layout_is_grid_with_6_panes_and_two_free_panes() {
    let resolved = resolve_runtime_args(&parse_args(&["multi-terminal"]), None).unwrap();
    assert_eq!(
        resolved.layout_mode,
        LayoutMode::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 6,
        }
    );
    assert_eq!(resolved.agents.len(), 6);
    assert!(resolved.agents[0].effective_command().is_none());
    assert_eq!(
        resolved.agents[1].effective_command().unwrap().program,
        "codex"
    );
    assert_eq!(
        resolved.agents[2].effective_command().unwrap().program,
        "kimi"
    );
    assert!(resolved.agents[3].effective_command().is_none());
    assert_eq!(
        resolved.agents[4].effective_command().unwrap().program,
        "opencode"
    );
    assert_eq!(
        resolved.agents[5].effective_command().unwrap().program,
        "kilo"
    );
}

#[test]
fn flag_layout_a_selects_layout_a() {
    let resolved =
        resolve_runtime_args(&parse_args(&["multi-terminal", "--layout", "a"]), None).unwrap();
    assert_eq!(resolved.layout_mode, LayoutMode::LegacyA);
}

#[test]
fn flag_layout_b_selects_layout_b() {
    let resolved =
        resolve_runtime_args(&parse_args(&["multi-terminal", "--layout", "b"]), None).unwrap();
    assert_eq!(resolved.layout_mode, LayoutMode::LegacyB);
}

#[test]
fn resolve_runtime_args_uses_saved_layout_agents_when_loading() {
    let args = parse_args(&["multi-terminal", "--load", "team"]);
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Legacy("a".to_string()),
        agents: vec![
            AgentConfig::new(AgentType::Custom("htop".to_string())).with_title("Monitor"),
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Custom("npm run dev".to_string())).with_title("App"),
            AgentConfig::new(AgentType::Cursor),
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
        "--no-cursor",
    ]);
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Legacy("b".to_string()),
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
        layout: multi_terminal::layout::SavedLayoutKind::Legacy("b".to_string()),
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
    let args = parse_args(&["multi-terminal", "--layout-type", "grid", "--panes", "6"]);

    assert_eq!(args.layout_type, Some(LayoutType::Grid));
    assert_eq!(args.pane_count, Some(6));
}

#[test]
fn parse_args_supports_positional_working_dir() {
    let args = parse_args(&["multi-terminal", "/tmp/demo", "--layout-type", "grid"]);

    assert_eq!(args.working_dir.as_deref(), Some(Path::new("/tmp/demo")));
    assert_eq!(args.layout_type, Some(LayoutType::Grid));
}

#[test]
fn resolve_working_dir_rejects_missing_path() {
    let path = std::env::temp_dir().join("multi-terminal-missing-dir-for-test");
    let error = resolve_working_dir(Some(path.as_path())).unwrap_err();

    assert!(error.contains("does not exist"));
}

#[test]
fn resolve_runtime_args_builds_dynamic_defaults() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "main-left",
        "--panes",
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
        "codex"
    );
    assert_eq!(
        resolved.agents[4].effective_command().unwrap().program,
        "kilo"
    );
}

#[test]
fn dynamic_defaults_assign_kilo_to_fifth_pane() {
    let args = parse_args(&["multi-terminal", "--layout-type", "grid", "--panes", "5"]);

    let resolved = resolve_runtime_args(&args, None).unwrap();

    assert_eq!(
        resolved.agents[4].effective_command().unwrap().program,
        "kilo"
    );
    assert_eq!(resolved.agents[4].effective_title(), "kilo");
}

#[test]
fn resolve_agents_applies_indexed_pane_override() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "grid",
        "--panes",
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
        "--panes",
        "3",
        "--pane",
        "4=lazygit",
    ]);

    let error = resolve_runtime_args(&args, None).unwrap_err();

    assert!(error.contains("pane index 4 is out of bounds for 3 panes"));
}

#[test]
fn resolve_runtime_args_uses_saved_dynamic_layout() {
    let args = parse_args(&["multi-terminal", "--load", "team"]);
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Dynamic {
            layout_type: LayoutType::MainTop,
            pane_count: 5,
        },
        agents: vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Claude),
            AgentConfig::new(AgentType::Codex),
            AgentConfig::new(AgentType::Cursor),
            AgentConfig::new(AgentType::Custom("npm test".to_string())),
        ],
        maximize: true,
    };

    let resolved = resolve_runtime_args(&args, Some(saved)).unwrap();

    assert_eq!(
        resolved.layout_mode,
        LayoutMode::Dynamic {
            layout_type: LayoutType::MainTop,
            pane_count: 5,
        }
    );
    assert_eq!(resolved.agents.len(), 5);
}

#[test]
fn resolve_runtime_args_uses_persisted_default_layout_when_no_cli_layout_is_provided() {
    let args = parse_args(&["multi-terminal"]);
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Dynamic {
            layout_type: LayoutType::MainLeft,
            pane_count: 5,
        },
        agents: vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Claude),
            AgentConfig::new(AgentType::Custom("npm run dev".to_string())).with_title("App"),
            AgentConfig::new(AgentType::Cursor),
            AgentConfig::new(AgentType::OpenCode),
        ],
        maximize: false,
    };

    let resolved =
        multi_terminal::resolve_runtime_args_with_defaults(&args, None, Some(saved)).unwrap();

    assert_eq!(
        resolved.layout_mode,
        LayoutMode::Dynamic {
            layout_type: LayoutType::MainLeft,
            pane_count: 5,
        }
    );
    assert_eq!(
        resolved.agents[2].effective_command().unwrap().program,
        "npm run dev"
    );
    assert_eq!(
        resolved.agents[4].effective_command().unwrap().program,
        "opencode"
    );
}

#[test]
fn cli_layout_overrides_persisted_default_layout() {
    let args = parse_args(&["multi-terminal", "--layout-type", "grid", "--panes", "3"]);
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Dynamic {
            layout_type: LayoutType::MainLeft,
            pane_count: 5,
        },
        agents: vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Claude),
            AgentConfig::new(AgentType::Codex),
            AgentConfig::new(AgentType::Cursor),
            AgentConfig::new(AgentType::OpenCode),
        ],
        maximize: false,
    };

    let resolved =
        multi_terminal::resolve_runtime_args_with_defaults(&args, None, Some(saved)).unwrap();

    assert_eq!(
        resolved.layout_mode,
        LayoutMode::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 3,
        }
    );
    assert_eq!(resolved.agents.len(), 3);
}

#[test]
fn loaded_layout_still_overrides_persisted_defaults() {
    let args = parse_args(&["multi-terminal", "--load", "team"]);
    let loaded = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Legacy("a".to_string()),
        agents: Layout::A.default_agents(),
        maximize: true,
    };
    let persisted = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 5,
        },
        agents: vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Claude),
            AgentConfig::new(AgentType::Codex),
            AgentConfig::new(AgentType::Cursor),
            AgentConfig::new(AgentType::OpenCode),
        ],
        maximize: false,
    };

    let resolved =
        multi_terminal::resolve_runtime_args_with_defaults(&args, Some(loaded), Some(persisted))
            .unwrap();

    assert_eq!(resolved.layout_mode, LayoutMode::LegacyA);
    assert_eq!(resolved.agents.len(), 4);
    assert!(resolved.maximize);
}

#[test]
fn no_opencode_disables_fifth_default_pane() {
    let args = parse_args(&[
        "multi-terminal",
        "--layout-type",
        "grid",
        "--panes",
        "5",
        "--no-opencode",
    ]);

    let resolved = resolve_runtime_args(&args, None).unwrap();

    assert!(resolved.agents[4].effective_command().is_none());
}

#[test]
fn args_parse_set_default_flag() {
    let args = parse_args(&["multi-terminal", "--set-default"]);

    assert!(args.set_default);
}

#[test]
fn args_parse_close_current_flags() {
    let primary = parse_args(&["multi-terminal", "--close-current"]);
    let alias = parse_args(&["multi-terminal", "--cc"]);

    assert!(primary.close_current);
    assert!(alias.close_current);
}

#[test]
fn args_parse_force_close_current_flags() {
    let primary = parse_args(&["multi-terminal", "--force-close-current"]);
    let alias = parse_args(&["multi-terminal", "--fcc"]);

    assert!(primary.force_close_current);
    assert!(alias.force_close_current);
}

#[test]
fn saved_layout_validate_accepts_legacy_shape() {
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Legacy("b".to_string()),
        agents: LayoutMode::LegacyB.default_agents(),
        maximize: false,
    };

    assert!(saved.validate().is_ok());
}

#[test]
fn pane_count_requires_layout_type() {
    let result = Args::try_parse_from(["multi-terminal", "--panes", "6"]);

    assert!(result.is_err());
}

#[test]
fn layout_conflicts_with_layout_type() {
    let result = Args::try_parse_from([
        "multi-terminal",
        "--layout",
        "a",
        "--layout-type",
        "grid",
        "--panes",
        "6",
    ]);

    assert!(result.is_err());
}

#[test]
fn saved_layout_rejects_dynamic_zero_panes() {
    let saved = SavedLayout {
        layout: multi_terminal::layout::SavedLayoutKind::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 0,
        },
        agents: vec![],
        maximize: false,
    };

    let error = saved.validate().unwrap_err();

    assert!(error.contains("pane count must be at least 1"));
}

#[test]
fn set_default_serializes_and_round_trips_correctly() {
    let runtime = multi_terminal::resolve_runtime_args(
        &parse_args(&[
            "multi-terminal",
            "--layout-type",
            "grid",
            "--panes",
            "5",
            "--pane",
            "2=npm run dev",
            "--title",
            "2=App",
        ]),
        None,
    )
    .unwrap();

    let saved = multi_terminal::layout::SavedLayout {
        layout: match runtime.layout_mode {
            LayoutMode::LegacyA => multi_terminal::layout::SavedLayoutKind::Legacy("a".to_string()),
            LayoutMode::LegacyB => multi_terminal::layout::SavedLayoutKind::Legacy("b".to_string()),
            LayoutMode::Dynamic {
                ref layout_type,
                pane_count,
            } => multi_terminal::layout::SavedLayoutKind::Dynamic {
                layout_type: layout_type.clone(),
                pane_count,
            },
        },
        agents: runtime.agents.clone(),
        maximize: runtime.maximize,
    };

    let serialized = serde_json::to_string_pretty(&saved).unwrap();
    let deserialized: multi_terminal::layout::SavedLayout =
        serde_json::from_str(&serialized).unwrap();

    assert!(deserialized.validate().is_ok());
    assert_eq!(deserialized.agents.len(), 5);
    assert_eq!(
        deserialized.agents[1].effective_command().unwrap().program,
        "npm run dev"
    );
    assert_eq!(deserialized.agents[1].effective_title(), "App");
}

#[test]
fn set_default_handles_corrupted_config_file_gracefully() {
    let path = SavedLayout::default_config_path();
    let parent = path.parent().unwrap().to_path_buf();

    std::fs::create_dir_all(&parent).ok();

    let backup = if path.exists() {
        let backup_path = path.with_extension("json.bak");
        std::fs::copy(&path, &backup_path).ok();
        Some(backup_path)
    } else {
        None
    };

    std::fs::write(&path, "{ invalid json }").ok();

    let result = SavedLayout::load_default();
    assert!(result.is_err());

    if let Some(backup_path) = backup {
        std::fs::copy(&backup_path, &path).ok();
        std::fs::remove_file(&backup_path).ok();
    } else if path.exists() {
        std::fs::remove_file(&path).ok();
    }
}
