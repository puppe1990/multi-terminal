pub mod iterm;
pub mod layout;
pub mod pty;
pub mod terminal_app;
pub mod tmux;

use clap::Parser;
use layout::Layout;

#[derive(Parser, Debug)]
#[command(
    name = "multi-terminal",
    about = "Abre 4 painéis de terminal com agentes de IA"
)]
pub struct Args {
    /// Layout dos painéis: a ou b (padrão: b)
    #[arg(long, value_parser = parse_layout, default_value = "b")]
    pub layout: Layout,
}

fn parse_layout(s: &str) -> Result<Layout, String> {
    match s.to_lowercase().as_str() {
        "a" => Ok(Layout::A),
        "b" => Ok(Layout::B),
        other => Err(format!("layout inválido '{}': use 'a' ou 'b'", other)),
    }
}

pub fn parse_args(args: &[&str]) -> Args {
    Args::parse_from(args)
}

pub fn run(args: Args) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    if cols < 80 || rows < 24 {
        eprintln!(
            "Erro: terminal muito pequeno ({}x{}). Mínimo: 80x24.",
            cols, rows
        );
        std::process::exit(1);
    }

    if crate::iterm::is_supported() {
        if let Err(e) = crate::iterm::run(&args.layout) {
            eprintln!("Erro no modo iTerm2: {}. Tentando tmux/PTY...", e);
        } else {
            return;
        }
    }

    match which::which("tmux") {
        Ok(_) => {
            if let Err(e) = crate::tmux::run(&args.layout) {
                eprintln!("Erro no modo tmux: {}. Tentando fallback PTY...", e);
                if let Err(e2) = crate::pty::run(&args.layout) {
                    eprintln!("Erro no fallback PTY: {}", e2);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            if let Err(e) = crate::pty::run(&args.layout) {
                eprintln!("Erro no modo PTY: {}", e);
                std::process::exit(1);
            }
        }
    }
}
