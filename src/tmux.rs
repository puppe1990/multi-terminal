use crate::layout::{AgentConfig, LayoutMode};

/// Builds the tmux shell command sequence for the given layout.
/// session_name: name of the tmux session to create.
pub fn build_commands(layout_mode: &LayoutMode, agents: &[AgentConfig], session_name: &str) -> Vec<String> {
    // Convert LayoutMode to Layout for now
    let layout = match layout_mode {
        LayoutMode::LegacyA => crate::layout::Layout::A,
        LayoutMode::LegacyB => crate::layout::Layout::B,
        LayoutMode::Dynamic { .. } => crate::layout::Layout::B, // Default to B for now
    };
    
    let mut cmds = vec![
        // Create new detached session
        format!(
            "tmux new-session -d -s {} -x \"$(tput cols)\" -y \"$(tput lines)\"",
            session_name
        ),
    ];

    match layout {
        crate::layout::Layout::B => {
            // Layout B: 2x2 symmetric
            // Split left/right
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Split left into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.0", session_name));
            // Split right into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
        }
        crate::layout::Layout::A => {
            // Layout A: left full height, right split top/bottom/bottom
            // Split left/right
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Split right into top/bottom
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
            // Split right-bottom into left/right (codex | qwen)
            cmds.push(format!("tmux split-window -h -t {}:0.2", session_name));
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
