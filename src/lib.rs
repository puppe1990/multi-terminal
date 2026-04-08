pub mod iterm;
pub mod layout;
pub mod pty;
pub mod terminal_app;
pub mod tmux;

use clap::Parser;
use layout::{AgentConfig, AgentType, Layout, SavedLayout};

#[derive(Debug, Clone)]
pub struct RuntimeArgs {
    pub layout: Layout,
    pub agents: Vec<AgentConfig>,
    pub maximize: bool,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "multi-terminal",
    about = "Opens 4 terminal panes with AI agents"
)]
pub struct Args {
    /// Pane layout: a or b (default: b)
    #[arg(long, value_parser = parse_layout, default_value = "b")]
    pub layout: Layout,

    /// Disable Claude agent
    #[arg(long)]
    pub no_claude: bool,

    /// Disable Codex agent
    #[arg(long)]
    pub no_codex: bool,

    /// Disable Qwen agent
    #[arg(long)]
    pub no_qwen: bool,

    /// Custom command for pane 1 (top-left or left)
    #[arg(long)]
    pub pane1: Option<String>,

    /// Custom command for pane 2 (top-right or right-top)
    #[arg(long)]
    pub pane2: Option<String>,

    /// Custom command for pane 3 (bottom-left or right-bottom-left)
    #[arg(long)]
    pub pane3: Option<String>,

    /// Custom command for pane 4 (bottom-right or right-bottom-right)
    #[arg(long)]
    pub pane4: Option<String>,

    /// Custom title for pane 1
    #[arg(long)]
    pub title1: Option<String>,

    /// Custom title for pane 2
    #[arg(long)]
    pub title2: Option<String>,

    /// Custom title for pane 3
    #[arg(long)]
    pub title3: Option<String>,

    /// Custom title for pane 4
    #[arg(long)]
    pub title4: Option<String>,

    /// Open maximized/fullscreen window
    #[arg(long)]
    pub maximize: bool,

    /// Save current configuration as a named layout
    #[arg(long)]
    pub save: Option<String>,

    /// Load a previously saved layout by name
    #[arg(long)]
    pub load: Option<String>,

    /// List all saved layouts
    #[arg(long)]
    pub list_layouts: bool,
}

fn parse_layout(s: &str) -> Result<Layout, String> {
    match s.to_lowercase().as_str() {
        "a" => Ok(Layout::A),
        "b" => Ok(Layout::B),
        other => Err(format!("invalid layout '{}': use 'a' or 'b'", other)),
    }
}

pub fn parse_args(args: &[&str]) -> Args {
    Args::parse_from(args)
}

pub fn resolve_agents(
    args: &Args,
    base_agents: Option<Vec<AgentConfig>>,
) -> Result<Vec<AgentConfig>, String> {
    let mut agents = base_agents.unwrap_or_else(|| args.layout.default_agents());

    if agents.len() != args.layout.expected_pane_count() {
        return Err(format!(
            "invalid agent configuration: expected {} panes, got {}",
            args.layout.expected_pane_count(),
            agents.len()
        ));
    }

    // Apply --no-* flags
    if args.no_claude {
        agents[1] = AgentConfig::new(AgentType::Shell);
    }
    if args.no_codex {
        agents[2] = AgentConfig::new(AgentType::Shell);
    }
    if args.no_qwen {
        agents[3] = AgentConfig::new(AgentType::Shell);
    }

    // Apply --paneN custom commands
    if let Some(cmd) = &args.pane1 {
        agents[0] = AgentConfig::new(AgentType::Custom(cmd.clone()))
            .with_title(args.title1.as_deref().unwrap_or(cmd));
    }
    if let Some(cmd) = &args.pane2 {
        agents[1] = AgentConfig::new(AgentType::Custom(cmd.clone()))
            .with_title(args.title2.as_deref().unwrap_or(cmd));
    }
    if let Some(cmd) = &args.pane3 {
        agents[2] = AgentConfig::new(AgentType::Custom(cmd.clone()))
            .with_title(args.title3.as_deref().unwrap_or(cmd));
    }
    if let Some(cmd) = &args.pane4 {
        agents[3] = AgentConfig::new(AgentType::Custom(cmd.clone()))
            .with_title(args.title4.as_deref().unwrap_or(cmd));
    }

    // Apply --titleN without --paneN
    if args.pane1.is_none() && args.title1.is_some() {
        agents[0].title = args.title1.clone();
    }
    if args.pane2.is_none() && args.title2.is_some() {
        agents[1].title = args.title2.clone();
    }
    if args.pane3.is_none() && args.title3.is_some() {
        agents[2].title = args.title3.clone();
    }
    if args.pane4.is_none() && args.title4.is_some() {
        agents[3].title = args.title4.clone();
    }

    Ok(agents)
}

pub fn resolve_runtime_args(
    args: &Args,
    saved: Option<SavedLayout>,
) -> Result<RuntimeArgs, String> {
    let (layout, base_agents, maximize) = match saved {
        Some(saved) => {
            saved.validate()?;
            let layout = parse_layout(&saved.layout)?;
            let maximize = saved.maximize || args.maximize;
            (layout, Some(saved.agents), maximize)
        }
        None => (args.layout.clone(), None, args.maximize),
    };

    let effective_args = Args {
        layout,
        maximize,
        ..args.clone()
    };

    let agents = resolve_agents(&effective_args, base_agents)?;

    Ok(RuntimeArgs {
        layout: effective_args.layout,
        agents,
        maximize: effective_args.maximize,
    })
}

pub fn run(args: Args) {
    // Handle --list-layouts
    if args.list_layouts {
        match SavedLayout::load_all() {
            Ok(layouts) if layouts.is_empty() => {
                println!("No saved layouts found.");
                return;
            }
            Ok(layouts) => {
                println!("Saved layouts:");
                for (name, layout) in layouts {
                    println!(
                        "  {} (layout: {}, maximize: {})",
                        name, layout.layout, layout.maximize
                    );
                }
                return;
            }
            Err(e) => {
                eprintln!("Error listing layouts: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Handle --load
    let runtime = if let Some(ref name) = args.load {
        match SavedLayout::load(name) {
            Ok(Some(saved)) => match resolve_runtime_args(&args, Some(saved)) {
                Ok(runtime) => runtime,
                Err(e) => {
                    eprintln!("Error loading layout: {}", e);
                    std::process::exit(1);
                }
            },
            Ok(None) => {
                eprintln!("Layout '{}' not found.", name);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error loading layout: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match resolve_runtime_args(&args, None) {
            Ok(runtime) => runtime,
            Err(e) => {
                eprintln!("Error resolving runtime configuration: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Handle --save
    if let Some(ref name) = args.save {
        let saved = SavedLayout {
            layout: match runtime.layout {
                Layout::A => "a".to_string(),
                Layout::B => "b".to_string(),
            },
            agents: runtime.agents.clone(),
            maximize: runtime.maximize,
        };

        if let Err(e) = saved.save(name) {
            eprintln!("Error saving layout: {}", e);
            std::process::exit(1);
        }
        println!("Layout '{}' saved successfully.", name);
        return;
    }

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    if cols < 80 || rows < 24 {
        eprintln!(
            "Error: terminal too small ({}x{}). Minimum: 80x24.",
            cols, rows
        );
        std::process::exit(1);
    }

    if crate::iterm::is_supported() {
        if let Err(e) = crate::iterm::run(&runtime.layout, &runtime.agents, runtime.maximize) {
            eprintln!("Error in iTerm2 mode: {}. Trying tmux/PTY...", e);
        } else {
            return;
        }
    } else if cfg!(target_os = "macos") {
        if let Err(e) = crate::iterm::ensure_installed() {
            eprintln!("Error installing iTerm2 automatically: {}", e);
        } else if let Err(e) = crate::iterm::run(&runtime.layout, &runtime.agents, runtime.maximize)
        {
            eprintln!(
                "iTerm2 installed but failed to open splits: {}. Trying tmux/PTY...",
                e
            );
        } else {
            return;
        }
    }

    match which::which("tmux") {
        Ok(_) => {
            if let Err(e) = crate::tmux::run(&runtime.layout, &runtime.agents) {
                eprintln!("Error in tmux mode: {}. Trying fallback PTY...", e);
                if let Err(e2) = crate::pty::run(&runtime.layout, &runtime.agents) {
                    eprintln!("Error in fallback PTY: {}", e2);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            if let Err(e) = crate::pty::run(&runtime.layout, &runtime.agents) {
                eprintln!("Error in PTY mode: {}", e);
                std::process::exit(1);
            }
        }
    }
}
