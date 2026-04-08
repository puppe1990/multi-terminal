pub mod layout;
pub mod tmux;
pub mod pty;

use clap::Parser;
use layout::Layout;

#[derive(Parser, Debug)]
#[command(name = "multi-terminal", about = "Abre 4 painéis de terminal com agentes de IA")]
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

fn main() {
    let args = Args::parse();
    run(args);
}

pub fn run(args: Args) {
    // implementado nos tasks seguintes
    let _ = args;
}
