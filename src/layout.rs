#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn to_shell_string(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaneConfig {
    pub command: Option<Command>,
}

impl PaneConfig {
    pub fn free() -> Self {
        Self { command: None }
    }

    pub fn with_command(program: &str, args: &[&str]) -> Self {
        Self {
            command: Some(Command::new(program, args)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    A,
    B,
}

impl Layout {
    /// Retorna os 4 painéis do layout em ordem:
    /// Layout B: [topo-esq(livre), topo-dir(claude), baixo-esq(codex), baixo-dir(qwen)]
    /// Layout A: [esq-total(livre), dir-topo(claude), dir-baixo-esq(codex), dir-baixo-dir(qwen)]
    pub fn panes(&self) -> Vec<PaneConfig> {
        vec![
            PaneConfig::free(),
            PaneConfig::with_command("claude", &["--dangerously-skip-permissions"]),
            PaneConfig::with_command("codex", &["--yolo"]),
            PaneConfig::with_command("qwen", &["--yolo"]),
        ]
    }
}
