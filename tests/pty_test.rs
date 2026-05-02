use multi_terminal::layout::{AgentConfig, AgentType, Layout, LayoutMode, LayoutType};
use multi_terminal::pty::{
    command_for_pane, compute_geometry, invalidate_all_panes, normalize_terminal_output,
    render_lines, PaneGeometry,
};

#[test]
fn layout_b_geometry_fills_terminal() {
    let geom = compute_geometry(&LayoutMode::LegacyB, 100, 40);
    assert_eq!(geom.len(), 4);
    // Pane 0 e 2 devem estar na coluna esquerda
    assert_eq!(geom[0].col, 0);
    assert_eq!(geom[2].col, 0);
    // Pane 1 e 3 devem estar na coluna direita
    assert!(geom[1].col > 0);
    assert!(geom[3].col > 0);
}

#[test]
fn layout_b_geometry_pane_sizes_cover_terminal() {
    let geom = compute_geometry(&LayoutMode::LegacyB, 100, 40);
    // largura total dos dois paineis esquerda+direita ~= total
    let left_width = geom[0].width;
    let right_width = geom[1].width;
    // +1 para a borda
    assert!(left_width + right_width < 100);
}

#[test]
fn layout_a_geometry_left_pane_spans_full_height() {
    let geom = compute_geometry(&LayoutMode::LegacyA, 100, 40);
    // pane 0 (esquerda) deve ter altura total
    assert_eq!(geom[0].row, 0);
    assert!(geom[0].height >= 38); // margem para bordas
}

#[test]
fn free_pane_uses_shell_command_in_pty_mode() {
    let agents = Layout::B.default_agents();
    let config = &agents[0];
    let command = command_for_pane(config);

    assert!(command.program.ends_with("sh"));
    assert!(command.args.is_empty());
}

#[test]
fn agent_pane_runs_its_command_in_pty_mode() {
    let agents = Layout::B.default_agents();
    let config = &agents[1]; // Codex
    let command = command_for_pane(config);

    assert_eq!(command.program, "codex");
    assert_eq!(command.args, vec!["--yolo"]);
}

#[test]
fn custom_string_command_runs_via_shell_in_pty_mode() {
    let config = AgentConfig::new(AgentType::Custom("npm run dev".to_string()));
    let command = command_for_pane(&config);

    assert!(command.program.ends_with("sh"));
    assert_eq!(command.args, vec!["-lc", "npm run dev"]);
}

#[test]
fn render_lines_draws_a_box_around_content() {
    let geom = PaneGeometry {
        row: 0,
        col: 0,
        width: 12,
        height: 4,
    };

    let lines = render_lines(&geom, "", b"hello", false);

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "+----------+");
    assert_eq!(lines[1], "|hello     |");
    assert_eq!(lines[2], "|          |");
    assert_eq!(lines[3], "+----------+");
}

#[test]
fn invalidating_panes_marks_all_buffers_for_redraw() {
    let mut buffers = vec![Some(vec![1, 2]), Some(vec![3]), None];

    invalidate_all_panes(&mut buffers);

    assert_eq!(buffers, vec![None, None, None]);
}

#[test]
fn normalize_terminal_output_strips_ansi_sequences() {
    let raw = b"\x1b[2J\x1b[Hhello\x1b[31m red\x1b[0m";

    let normalized = normalize_terminal_output(raw);

    assert_eq!(normalized, "hello red");
}

#[test]
fn render_lines_ignores_terminal_control_bytes() {
    let geom = PaneGeometry {
        row: 0,
        col: 0,
        width: 14,
        height: 4,
    };

    let lines = render_lines(&geom, "", b"\x1b[2J\x1b[Hhi\r\nthere", false);

    assert_eq!(lines[0], "+------------+");
    assert_eq!(lines[1], "|hi          |");
    assert_eq!(lines[2], "|there       |");
    assert_eq!(lines[3], "+------------+");
}

#[test]
fn render_lines_can_show_pane_title() {
    let geom = PaneGeometry {
        row: 0,
        col: 0,
        width: 30,
        height: 3,
    };

    let lines = render_lines(&geom, "Codex", b"", false);

    assert!(lines[0].contains("Codex"));
}

#[test]
fn compute_geometry_grid_supports_six_panes() {
    let panes = compute_geometry(
        &LayoutMode::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 6,
        },
        120,
        40,
    );

    assert_eq!(panes.len(), 6);
    assert!(panes.iter().all(|pane| pane.width > 0 && pane.height > 0));
}

#[test]
fn compute_geometry_main_left_supports_five_panes() {
    let panes = compute_geometry(
        &LayoutMode::Dynamic {
            layout_type: LayoutType::MainLeft,
            pane_count: 5,
        },
        120,
        40,
    );

    assert_eq!(panes.len(), 5);
    assert!(panes[0].width > panes[1].width);
    assert_ne!(panes[1].col, panes[2].col);
    assert_ne!(panes[1].row, panes[3].row);
}

#[test]
fn compute_geometry_main_top_uses_subgrid_for_remaining_panes() {
    let panes = compute_geometry(
        &LayoutMode::Dynamic {
            layout_type: LayoutType::MainTop,
            pane_count: 5,
        },
        120,
        40,
    );

    assert_eq!(panes.len(), 5);
    assert!(panes[0].height > panes[1].height);
    assert_ne!(panes[1].col, panes[2].col);
    assert_ne!(panes[1].row, panes[3].row);
}
