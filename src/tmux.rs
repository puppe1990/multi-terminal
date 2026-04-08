use crate::layout::{AgentConfig, LayoutMode};

/// Builds the tmux shell command sequence for the given layout.
/// session_name: name of the tmux session to create.
pub fn build_commands(layout_mode: &LayoutMode, agents: &[AgentConfig], session_name: &str) -> Vec<String> {
    let mut cmds = vec![
        // Create new detached session
        format!(
            "tmux new-session -d -s {} -x \"$(tput cols)\" -y \"$(tput lines)\"",
            session_name
        ),
    ];

    let _pane_count = layout_mode.pane_count();
    
    // Generate splits based on layout mode
    match layout_mode {
        LayoutMode::LegacyA => {
            // Layout A: left full height, right split top/bottom/bottom
            // Split left/right
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Split right into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
            // Split right-bottom into left/right (codex | qwen)
            cmds.push(format!("tmux split-window -h -t {}:0.2", session_name));
        }
        LayoutMode::LegacyB => {
            // Layout B: 2x2 symmetric
            // Split left/right
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Split left into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.0", session_name));
            // Split right into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
        }
        LayoutMode::Dynamic {
            layout_type,
            pane_count,
        } => {
            build_dynamic_splits(&mut cmds, session_name, layout_type, *pane_count);
        }
    }

    // Send commands to each pane
    for (index, agent) in agents.iter().enumerate() {
        if let Some(cmd) = agent.effective_command() {
            cmds.push(format!(
                "tmux send-keys -t {}:0.{} '{}' Enter",
                session_name,
                index,
                cmd.to_shell_string()
            ));
        }
    }

    // Select pane 0 and attach
    cmds.push(format!("tmux select-pane -t {}:0.0", session_name));
    cmds.push(format!("tmux attach-session -t {}", session_name));

    cmds
}

fn build_dynamic_splits(
    cmds: &mut Vec<String>,
    session_name: &str,
    layout_type: &crate::layout::LayoutType,
    pane_count: usize,
) {
    if pane_count <= 1 {
        return; // No splits needed
    }

    match layout_type {
        crate::layout::LayoutType::Grid => {
            // Grid: split in a balanced way
            // For now, use a simple iterative approach
            let num_cols = (pane_count as f64).sqrt().ceil() as usize;
            let num_rows = ((pane_count + num_cols - 1) / num_cols) as usize;

            // First, create columns by splitting horizontally
            for _col in 1..num_cols {
                cmds.push(format!(
                    "tmux split-window -h -t {}:0.0",
                    session_name
                ));
            }

            // Then split each column into rows vertically
            for col in 0..num_cols {
                for row in 1..num_rows {
                    let pane_idx = col + (row - 1) * num_cols;
                    if pane_idx < pane_count - 1 {
                        cmds.push(format!(
                            "tmux split-window -v -t {}:0.{}",
                            session_name, pane_idx
                        ));
                    }
                }
            }
        }
        crate::layout::LayoutType::MainLeft => {
            // Main left: first split vertical, then split the right side
            cmds.push(format!(
                "tmux split-window -h -t {}:0.0",
                session_name
            ));
            
            // Split the right side into remaining panes
            for i in 1..pane_count - 1 {
                cmds.push(format!(
                    "tmux split-window -v -t {}:0.{}",
                    session_name, i
                ));
            }
        }
        crate::layout::LayoutType::MainTop => {
            // Main top: first split horizontal, then split the bottom side
            cmds.push(format!(
                "tmux split-window -v -t {}:0.0",
                session_name
            ));
            
            // Split the bottom side into remaining panes
            for i in 1..pane_count - 1 {
                cmds.push(format!(
                    "tmux split-window -h -t {}:0.{}",
                    session_name, i
                ));
            }
        }
    }
}

/// Executes the tmux command sequence for the given layout.
/// Returns Err with message if any command fails (except kill-session).
pub fn run(layout_mode: &LayoutMode, agents: &[AgentConfig]) -> Result<(), String> {
    let session = "multi-terminal";

    // Kill session if exists (fire-and-forget)
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tmux kill-session -t {} 2>/dev/null; true",
            session
        ))
        .status();

    let commands = build_commands(layout_mode, agents, session);

    for cmd in &commands {
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| format!("failed to execute '{}': {}", cmd, e))?;

        // attach-session takes over terminal — if it fails, it's a real error
        if !status.success() {
            return Err(format!("tmux command failed: {}", cmd));
        }
    }

    Ok(())
}
