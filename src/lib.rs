pub mod current_terminal;
pub mod iterm;
pub mod layout;
pub mod pty;
pub mod terminal_app;
pub mod tmux;

use clap::{Parser, ValueHint};
use layout::{AgentConfig, AgentType, Command, Layout, LayoutMode, LayoutType, SavedLayout};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RuntimeArgs {
    pub layout_mode: LayoutMode,
    pub agents: Vec<AgentConfig>,
    pub maximize: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneOverride {
    pub index: usize,
    pub value: String,
}

fn parse_pane_override(s: &str) -> Result<PaneOverride, String> {
    let (index, value) = s
        .split_once('=')
        .ok_or_else(|| "expected INDEX=VALUE".to_string())?;
    let index = index
        .parse::<usize>()
        .map_err(|_| format!("invalid pane index '{}'", index))?;
    if index == 0 {
        return Err("pane index must start at 1".to_string());
    }
    Ok(PaneOverride {
        index,
        value: value.to_string(),
    })
}

#[derive(Parser, Debug, Clone)]
#[command(name = "multi-terminal", about = "Opens terminal panes with AI agents")]
pub struct Args {
    /// Working directory to open in every pane
    #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub working_dir: Option<PathBuf>,

    /// Pane layout: a or b (default: b)
    #[arg(long, value_parser = parse_layout, conflicts_with = "layout_type")]
    pub layout: Option<Layout>,

    /// Layout type for dynamic panes: grid, main-left, main-top
    #[arg(long, value_parser = parse_layout_type, conflicts_with = "layout")]
    pub layout_type: Option<LayoutType>,

    /// Number of panes for dynamic layouts (default: 5)
    #[arg(long = "panes", alias = "pane-count", requires = "layout_type")]
    pub pane_count: Option<usize>,

    /// Disable Claude agent
    #[arg(long)]
    pub no_claude: bool,

    /// Disable Codex agent
    #[arg(long)]
    pub no_codex: bool,

    /// Disable Cursor agent
    #[arg(long, alias = "no-qwen")]
    pub no_cursor: bool,

    /// Disable OpenCode agent
    #[arg(long)]
    pub no_opencode: bool,

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

    /// Open maximized/fullscreen window (default: true)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub maximize: bool,

    /// Don't maximize the window
    #[arg(long, action = clap::ArgAction::SetTrue, overrides_with = "maximize")]
    pub no_maximize: bool,

    /// Save current configuration as a named layout
    #[arg(long)]
    pub save: Option<String>,

    /// Persist the resolved configuration as the default startup layout
    #[arg(long)]
    pub set_default: bool,

    /// Close the current terminal window after opening the new layout
    #[arg(long, alias = "cc")]
    pub close_current: bool,

    /// Force close the current terminal window without confirmation
    #[arg(long, alias = "fcc", conflicts_with = "close_current")]
    pub force_close_current: bool,

    /// Load a previously saved layout by name
    #[arg(long)]
    pub load: Option<String>,

    /// List all saved layouts
    #[arg(long)]
    pub list_layouts: bool,

    /// Override command for pane INDEX (1-based): --pane INDEX=COMMAND
    #[arg(long = "pane", value_parser = parse_pane_override)]
    pub pane_overrides: Vec<PaneOverride>,

    /// Override title for pane INDEX (1-based): --title INDEX=TITLE
    #[arg(long = "title", value_parser = parse_pane_override)]
    pub title_overrides: Vec<PaneOverride>,
}

fn parse_layout(s: &str) -> Result<Layout, String> {
    match s.to_lowercase().as_str() {
        "a" => Ok(Layout::A),
        "b" => Ok(Layout::B),
        other => Err(format!("invalid layout '{}': use 'a' or 'b'", other)),
    }
}

fn parse_layout_type(s: &str) -> Result<LayoutType, String> {
    match s.to_lowercase().as_str() {
        "grid" => Ok(LayoutType::Grid),
        "main-left" => Ok(LayoutType::MainLeft),
        "main-top" => Ok(LayoutType::MainTop),
        other => Err(format!(
            "invalid layout type '{}': use 'grid', 'main-left' or 'main-top'",
            other
        )),
    }
}

pub fn parse_args(args: &[&str]) -> Args {
    Args::parse_from(args)
}

fn hardcoded_startup_default() -> (LayoutMode, Vec<AgentConfig>) {
    (
        LayoutMode::Dynamic {
            layout_type: LayoutType::Grid,
            pane_count: 6,
        },
        vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Codex),
            AgentConfig::new(AgentType::Custom("kimi".to_string()))
                .with_command(Command::new("kimi", &["--yolo"])),
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::OpenCode),
            AgentConfig::new(AgentType::Custom("kilo".to_string())),
        ],
    )
}

pub fn resolve_agents(
    args: &Args,
    base_agents: Option<Vec<AgentConfig>>,
) -> Result<Vec<AgentConfig>, String> {
    let layout = args.layout.clone().unwrap_or(Layout::B);
    let mut agents = base_agents.unwrap_or_else(|| layout.default_agents());

    if agents.len() != layout.expected_pane_count() {
        return Err(format!(
            "invalid agent configuration: expected {} panes, got {}",
            layout.expected_pane_count(),
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
    if args.no_cursor {
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

pub fn resolve_agents_dynamic(
    args: &Args,
    layout_mode: &LayoutMode,
    base_agents: Option<Vec<AgentConfig>>,
) -> Result<Vec<AgentConfig>, String> {
    let pane_count = layout_mode.pane_count();
    let mut agents = base_agents.unwrap_or_else(|| layout_mode.default_agents());

    if agents.len() != pane_count {
        return Err(format!(
            "invalid agent configuration: expected {} panes, got {}",
            pane_count,
            agents.len()
        ));
    }

    // Apply --no-* flags (only for first 4 panes in legacy mode)
    if pane_count > 1 && args.no_claude {
        agents[1] = AgentConfig::new(AgentType::Shell);
    }
    if pane_count > 2 && args.no_codex {
        agents[2] = AgentConfig::new(AgentType::Shell);
    }
    if pane_count > 3 && args.no_cursor {
        agents[3] = AgentConfig::new(AgentType::Shell);
    }
    if pane_count > 4 && args.no_opencode {
        agents[4] = AgentConfig::new(AgentType::Shell);
    }

    // Apply --paneN custom commands (only for first 4 panes)
    if let Some(cmd) = &args.pane1 {
        if pane_count > 0 {
            agents[0] = AgentConfig::new(AgentType::Custom(cmd.clone()))
                .with_title(args.title1.as_deref().unwrap_or(cmd));
        }
    }
    if let Some(cmd) = &args.pane2 {
        if pane_count > 1 {
            agents[1] = AgentConfig::new(AgentType::Custom(cmd.clone()))
                .with_title(args.title2.as_deref().unwrap_or(cmd));
        }
    }
    if let Some(cmd) = &args.pane3 {
        if pane_count > 2 {
            agents[2] = AgentConfig::new(AgentType::Custom(cmd.clone()))
                .with_title(args.title3.as_deref().unwrap_or(cmd));
        }
    }
    if let Some(cmd) = &args.pane4 {
        if pane_count > 3 {
            agents[3] = AgentConfig::new(AgentType::Custom(cmd.clone()))
                .with_title(args.title4.as_deref().unwrap_or(cmd));
        }
    }

    // Apply --titleN without --paneN
    if args.pane1.is_none() && args.title1.is_some() && pane_count > 0 {
        agents[0].title = args.title1.clone();
    }
    if args.pane2.is_none() && args.title2.is_some() && pane_count > 1 {
        agents[1].title = args.title2.clone();
    }
    if args.pane3.is_none() && args.title3.is_some() && pane_count > 2 {
        agents[2].title = args.title3.clone();
    }
    if args.pane4.is_none() && args.title4.is_some() && pane_count > 3 {
        agents[3].title = args.title4.clone();
    }

    // Apply indexed pane overrides
    for override_item in &args.pane_overrides {
        let idx = override_item.index - 1; // Convert 1-based to 0-based
        if idx >= pane_count {
            return Err(format!(
                "pane index {} is out of bounds for {} panes",
                override_item.index, pane_count
            ));
        }
        agents[idx] = AgentConfig::new(AgentType::Custom(override_item.value.clone()));
    }

    // Apply indexed title overrides
    for override_item in &args.title_overrides {
        let idx = override_item.index - 1; // Convert 1-based to 0-based
        if idx >= pane_count {
            return Err(format!(
                "pane index {} is out of bounds for {} panes",
                override_item.index, pane_count
            ));
        }
        agents[idx].title = Some(override_item.value.clone());
    }

    Ok(agents)
}

fn args_define_layout(args: &Args) -> bool {
    args.layout.is_some() || args.layout_type.is_some()
}

fn resolve_maximize(args: &Args, base_maximize: Option<bool>) -> bool {
    if args.no_maximize {
        false
    } else if args.maximize {
        true
    } else {
        base_maximize.unwrap_or(true)
    }
}

pub fn resolve_runtime_args_with_defaults(
    args: &Args,
    saved: Option<SavedLayout>,
    persisted_default: Option<SavedLayout>,
) -> Result<RuntimeArgs, String> {
    let saved = match saved {
        Some(saved) => Some(saved),
        None if !args_define_layout(args) => persisted_default,
        None => None,
    };

    let (layout_mode, base_agents, maximize) = match saved {
        Some(saved) => {
            saved.validate()?;
            let layout_mode = saved.to_layout_mode()?;
            let maximize = resolve_maximize(args, Some(saved.maximize));
            (layout_mode, Some(saved.agents), maximize)
        }
        None => {
            let maximize = resolve_maximize(args, None);

            // Determine layout mode from CLI args
            let (layout_mode, base_agents) = if let Some(layout_type) = &args.layout_type {
                let pane_count = args.pane_count.unwrap_or(5).max(1);
                (
                    LayoutMode::Dynamic {
                        layout_type: layout_type.clone(),
                        pane_count,
                    },
                    None,
                )
            } else if let Some(layout) = &args.layout {
                // Legacy --layout flag still supported
                (
                    match layout.clone() {
                        Layout::A => LayoutMode::LegacyA,
                        Layout::B => LayoutMode::LegacyB,
                    },
                    None,
                )
            } else {
                let (layout_mode, agents) = hardcoded_startup_default();
                (layout_mode, Some(agents))
            };

            (layout_mode, base_agents, maximize)
        }
    };

    // Build effective Args for agent resolution
    let effective_args = Args {
        maximize,
        ..args.clone()
    };

    let agents = resolve_agents_dynamic(&effective_args, &layout_mode, base_agents)?;

    Ok(RuntimeArgs {
        layout_mode,
        agents,
        maximize: effective_args.maximize,
    })
}

pub fn resolve_runtime_args(
    args: &Args,
    saved: Option<SavedLayout>,
) -> Result<RuntimeArgs, String> {
    resolve_runtime_args_with_defaults(args, saved, None)
}

fn saved_layout_from_runtime(runtime: &RuntimeArgs) -> SavedLayout {
    let layout_kind = match runtime.layout_mode {
        LayoutMode::LegacyA => layout::SavedLayoutKind::Legacy("a".to_string()),
        LayoutMode::LegacyB => layout::SavedLayoutKind::Legacy("b".to_string()),
        LayoutMode::Dynamic {
            ref layout_type,
            pane_count,
        } => layout::SavedLayoutKind::Dynamic {
            layout_type: layout_type.clone(),
            pane_count,
        },
    };

    SavedLayout {
        layout: layout_kind,
        agents: runtime.agents.clone(),
        maximize: runtime.maximize,
    }
}

pub fn validate_fallback_terminal_size(cols: u16, rows: u16) -> Result<(), String> {
    if cols < 80 || rows < 24 {
        Err(format!(
            "terminal too small ({}x{}). Minimum: 80x24.",
            cols, rows
        ))
    } else {
        Ok(())
    }
}

pub fn resolve_working_dir(path: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let Some(path) = path else {
        return Ok(None);
    };

    if !path.exists() {
        return Err(format!(
            "working directory '{}' does not exist",
            path.display()
        ));
    }

    if !path.is_dir() {
        return Err(format!(
            "working directory '{}' is not a directory",
            path.display()
        ));
    }

    std::fs::canonicalize(path).map(Some).map_err(|e| {
        format!(
            "failed to resolve working directory '{}': {e}",
            path.display()
        )
    })
}

fn ensure_fallback_terminal_size() {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    if let Err(e) = validate_fallback_terminal_size(cols, rows) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
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
                    let layout_desc = match layout.layout {
                        layout::SavedLayoutKind::Legacy(ref s) => s.clone(),
                        layout::SavedLayoutKind::Dynamic {
                            ref layout_type,
                            pane_count,
                        } => {
                            let type_str = match layout_type {
                                LayoutType::Grid => "grid",
                                LayoutType::MainLeft => "main-left",
                                LayoutType::MainTop => "main-top",
                            };
                            format!("{} {} panes", type_str, pane_count)
                        }
                    };
                    println!(
                        "  {} (layout: {}, maximize: {})",
                        name, layout_desc, layout.maximize
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

    let persisted_default = match SavedLayout::load_default() {
        Ok(saved) => saved,
        Err(e) => {
            eprintln!(
                "Warning: could not load default configuration, using defaults: {}",
                e
            );
            None
        }
    };

    let loaded_layout = if let Some(ref name) = args.load {
        match SavedLayout::load(name) {
            Ok(Some(saved)) => Some(saved),
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
        None
    };

    let runtime = match resolve_runtime_args_with_defaults(&args, loaded_layout, persisted_default)
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("Error resolving runtime configuration: {}", e);
            std::process::exit(1);
        }
    };

    if args.set_default {
        let saved = saved_layout_from_runtime(&runtime);

        if let Err(e) = saved.save_default() {
            eprintln!("Error saving default configuration: {}", e);
            std::process::exit(1);
        }
        println!("Default configuration saved successfully.");
        return;
    }

    // Handle --save
    if let Some(ref name) = args.save {
        let saved = saved_layout_from_runtime(&runtime);

        if let Err(e) = saved.save(name) {
            eprintln!("Error saving layout: {}", e);
            std::process::exit(1);
        }
        println!("Layout '{}' saved successfully.", name);
        return;
    }

    let working_dir = match resolve_working_dir(args.working_dir.as_deref()) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving working directory: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(path) = working_dir {
        if let Err(e) = std::env::set_current_dir(&path) {
            eprintln!(
                "Error changing working directory to '{}': {}",
                path.display(),
                e
            );
            std::process::exit(1);
        }
    }

    let should_close_current = args.close_current || args.force_close_current;
    let close_request = crate::current_terminal::capture_close_request(should_close_current)
        .map_err(|error| eprintln!("Warning: {error}"))
        .ok()
        .flatten();

    if crate::iterm::is_supported() {
        if let Err(e) = crate::iterm::run(&runtime.layout_mode, &runtime.agents, runtime.maximize) {
            eprintln!("Error in iTerm2 mode: {}. Trying tmux/PTY...", e);
        } else {
            crate::current_terminal::close_if_requested(
                close_request.as_ref(),
                args.force_close_current,
            );
            return;
        }
    } else if cfg!(target_os = "macos") {
        if let Err(e) = crate::iterm::ensure_installed() {
            eprintln!("Error installing iTerm2 automatically: {}", e);
        } else if let Err(e) =
            crate::iterm::run(&runtime.layout_mode, &runtime.agents, runtime.maximize)
        {
            eprintln!(
                "iTerm2 installed but failed to open splits: {}. Trying tmux/PTY...",
                e
            );
        } else {
            crate::current_terminal::close_if_requested(
                close_request.as_ref(),
                args.force_close_current,
            );
            return;
        }
    }

    if should_close_current {
        eprintln!(
            "Warning: {} requires launching a separate terminal window.",
            if args.force_close_current {
                "--force-close-current"
            } else {
                "--close-current"
            }
        );
    }

    ensure_fallback_terminal_size();

    match which::which("tmux") {
        Ok(_) => {
            if let Err(e) = crate::tmux::run(&runtime.layout_mode, &runtime.agents) {
                eprintln!("Error in tmux mode: {}. Trying fallback PTY...", e);
                if let Err(e2) = crate::pty::run(&runtime.layout_mode, &runtime.agents) {
                    eprintln!("Error in fallback PTY: {}", e2);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            if let Err(e) = crate::pty::run(&runtime.layout_mode, &runtime.agents) {
                eprintln!("Error in PTY mode: {}", e);
                std::process::exit(1);
            }
        }
    }
}
