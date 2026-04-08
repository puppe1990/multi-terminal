use crate::layout::Layout;

/// Constrói a sequência de comandos tmux shell para o layout dado.
/// session_name: nome da sessão tmux a criar.
pub fn build_commands(layout: &Layout, session_name: &str) -> Vec<String> {
    let mut cmds = vec![
        // Cria nova sessão detached
        format!(
            "tmux new-session -d -s {} -x \"$(tput cols)\" -y \"$(tput lines)\"",
            session_name
        ),
    ];

    match layout {
        Layout::B => {
            // Layout B: 2x2 simétrico
            // Divide em esq/dir
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Divide esq em cima/baixo
            cmds.push(format!("tmux split-window -v -t {}:0.0", session_name));
            // Divide dir em cima/baixo
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
            // Envia comandos: pane 1=claude, pane 2=codex, pane 3=qwen
            // (pane 0 fica livre)
            cmds.push(format!(
                "tmux send-keys -t {}:0.1 'claude --dangerously-skip-permissions' Enter",
                session_name
            ));
            cmds.push(format!(
                "tmux send-keys -t {}:0.2 'codex --yolo' Enter",
                session_name
            ));
            cmds.push(format!(
                "tmux send-keys -t {}:0.3 'qwen --yolo' Enter",
                session_name
            ));
        }
        Layout::A => {
            // Layout A: esq ocupa altura total, dir divide em cima/baixo/baixo
            // Divide em esq/dir
            cmds.push(format!("tmux split-window -h -t {}:0.0", session_name));
            // Divide dir em cima/baixo
            cmds.push(format!("tmux split-window -v -t {}:0.1", session_name));
            // Divide dir-baixo em esq/dir (codex | qwen)
            cmds.push(format!("tmux split-window -h -t {}:0.2", session_name));
            // Envia comandos: pane 1=claude, pane 2=codex, pane 3=qwen
            cmds.push(format!(
                "tmux send-keys -t {}:0.1 'claude --dangerously-skip-permissions' Enter",
                session_name
            ));
            cmds.push(format!(
                "tmux send-keys -t {}:0.2 'codex --yolo' Enter",
                session_name
            ));
            cmds.push(format!(
                "tmux send-keys -t {}:0.3 'qwen --yolo' Enter",
                session_name
            ));
        }
    }

    // Seleciona pane 0 (livre) e faz attach
    cmds.push(format!("tmux select-pane -t {}:0.0", session_name));
    cmds.push(format!("tmux attach-session -t {}", session_name));

    cmds
}

/// Executa a sequência de comandos tmux para o layout dado.
/// Retorna Err com mensagem se algum comando falhar (exceto kill-session).
pub fn run(layout: &Layout) -> Result<(), String> {
    let session = "multi-terminal";

    // Kill session se existir (fire-and-forget)
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tmux kill-session -t {} 2>/dev/null; true",
            session
        ))
        .status();

    let commands = build_commands(layout, session);

    for cmd in &commands {
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| format!("falha ao executar '{}': {}", cmd, e))?;

        // attach-session assume controle do terminal — se falhar, é erro real
        if !status.success() {
            return Err(format!("comando tmux falhou: {}", cmd));
        }
    }

    Ok(())
}
