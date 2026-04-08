use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    Claude,
    Codex,
    Qwen,
    OpenCode,
    Shell,
    Custom(String),
}

impl AgentType {
    pub fn default_command(&self) -> Option<Command> {
        match self {
            AgentType::Claude => Some(Command::new("claude", &["--dangerously-skip-permissions"])),
            AgentType::Codex => Some(Command::new("codex", &["--yolo"])),
            AgentType::Qwen => Some(Command::new("qwen", &["--yolo"])),
            AgentType::OpenCode => Some(Command::new("opencode", &[])),
            AgentType::Shell => None,
            AgentType::Custom(cmd) => Some(Command::new(cmd, &[])),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            AgentType::Claude => "claude",
            AgentType::Codex => "codex",
            AgentType::Qwen => "qwen",
            AgentType::OpenCode => "opencode",
            AgentType::Shell => "shell",
            AgentType::Custom(cmd) => cmd,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "type")]
    pub agent_type: AgentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl AgentConfig {
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            command: None,
            title: None,
        }
    }

    pub fn with_command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn effective_command(&self) -> Option<Command> {
        self.command
            .clone()
            .or_else(|| self.agent_type.default_command())
    }

    pub fn effective_title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| match &self.agent_type {
                AgentType::Claude => "Claude AI".to_string(),
                AgentType::Codex => "Codex".to_string(),
                AgentType::Qwen => "Qwen".to_string(),
                AgentType::OpenCode => "OpenCode".to_string(),
                AgentType::Shell => "Shell".to_string(),
                AgentType::Custom(cmd) => cmd.clone(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    A,
    B,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Grid,
    MainLeft,
    MainTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitOperation {
    pub parent: usize,
    pub new_index: usize,
    pub direction: SplitDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutMode {
    LegacyA,
    LegacyB,
    Dynamic {
        layout_type: LayoutType,
        pane_count: usize,
    },
}

impl LayoutMode {
    pub fn pane_count(&self) -> usize {
        match self {
            LayoutMode::LegacyA | LayoutMode::LegacyB => 4,
            LayoutMode::Dynamic { pane_count, .. } => *pane_count,
        }
    }

    pub fn default_agents(&self) -> Vec<AgentConfig> {
        let count = self.pane_count();
        let mut agents = vec![AgentConfig::new(AgentType::Shell); count];

        // Assign default agents based on position
        if count > 1 {
            agents[1] = AgentConfig::new(AgentType::Claude);
        }
        if count > 2 {
            agents[2] = AgentConfig::new(AgentType::Codex);
        }
        if count > 3 {
            agents[3] = AgentConfig::new(AgentType::Qwen);
        }
        if count > 4 {
            agents[4] = AgentConfig::new(AgentType::OpenCode);
        }

        agents
    }

    pub fn split_operations(&self) -> Vec<SplitOperation> {
        match self {
            LayoutMode::LegacyA => vec![
                SplitOperation {
                    parent: 0,
                    new_index: 1,
                    direction: SplitDirection::Horizontal,
                },
                SplitOperation {
                    parent: 1,
                    new_index: 2,
                    direction: SplitDirection::Vertical,
                },
                SplitOperation {
                    parent: 2,
                    new_index: 3,
                    direction: SplitDirection::Horizontal,
                },
            ],
            LayoutMode::LegacyB => vec![
                SplitOperation {
                    parent: 0,
                    new_index: 1,
                    direction: SplitDirection::Horizontal,
                },
                SplitOperation {
                    parent: 0,
                    new_index: 2,
                    direction: SplitDirection::Vertical,
                },
                SplitOperation {
                    parent: 1,
                    new_index: 3,
                    direction: SplitDirection::Vertical,
                },
            ],
            LayoutMode::Dynamic {
                layout_type,
                pane_count,
            } => dynamic_split_operations(layout_type, *pane_count),
        }
    }
}

impl Layout {
    /// Returns the canonical pane positions for this layout
    /// Layout B (2x2 grid): [top-left, top-right, bottom-left, bottom-right]
    /// Layout A (left + right split): [left-full, right-top, right-bottom-left, right-bottom-right]
    pub fn pane_positions(&self) -> Vec<&'static str> {
        match self {
            Layout::B => vec!["top-left", "top-right", "bottom-left", "bottom-right"],
            Layout::A => vec![
                "left",
                "right-top",
                "right-bottom-left",
                "right-bottom-right",
            ],
        }
    }

    /// Default agents for this layout in pane order
    pub fn default_agents(&self) -> Vec<AgentConfig> {
        vec![
            AgentConfig::new(AgentType::Shell),
            AgentConfig::new(AgentType::Claude),
            AgentConfig::new(AgentType::Codex),
            AgentConfig::new(AgentType::Qwen),
        ]
    }

    pub fn panes(&self, agents: &[AgentConfig]) -> Vec<AgentConfig> {
        agents.to_vec()
    }

    pub fn expected_pane_count(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SavedLayoutKind {
    Legacy(String),
    Dynamic {
        layout_type: LayoutType,
        pane_count: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    pub layout: SavedLayoutKind,
    pub agents: Vec<AgentConfig>,
    pub maximize: bool,
}

impl SavedLayout {
    pub fn config_path() -> std::path::PathBuf {
        Self::config_path_for("layouts.json")
    }

    pub fn default_config_path() -> std::path::PathBuf {
        Self::config_path_for("default.json")
    }

    fn config_path_for(file_name: &str) -> std::path::PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("multi-terminal");
        path.push(file_name);
        path
    }

    pub fn load_all() -> Result<Vec<(String, SavedLayout)>, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read layout config: {}", e))?;

        let map: std::collections::HashMap<String, SavedLayout> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse layout config: {}", e))?;

        Ok(map.into_iter().collect())
    }

    pub fn load(name: &str) -> Result<Option<SavedLayout>, String> {
        let layouts = Self::load_all()?;
        Ok(layouts.into_iter().find(|(n, _)| n == name).map(|(_, l)| l))
    }

    pub fn load_default() -> Result<Option<SavedLayout>, String> {
        let path = Self::default_config_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read default config: {}", e))?;

        let saved: SavedLayout = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse default config: {}", e))?;

        Ok(Some(saved))
    }

    pub fn save(&self, name: &str) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let mut map: std::collections::HashMap<String, SavedLayout> = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read existing config: {}", e))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        map.insert(name.to_string(), self.clone());

        let content = serde_json::to_string_pretty(&map)
            .map_err(|e| format!("Failed to serialize layout config: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write layout config: {}", e))?;

        Ok(())
    }

    pub fn save_default(&self) -> Result<(), String> {
        let path = Self::default_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize default config: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write default config: {}", e))?;

        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        let layout_mode = self.to_layout_mode()?;

        let expected = layout_mode.pane_count();
        if self.agents.len() != expected {
            return Err(format!(
                "invalid saved layout: expected {} panes, got {}",
                expected,
                self.agents.len()
            ));
        }

        Ok(())
    }

    pub fn to_layout_mode(&self) -> Result<LayoutMode, String> {
        match &self.layout {
            SavedLayoutKind::Legacy(s) => match s.to_lowercase().as_str() {
                "a" => Ok(LayoutMode::LegacyA),
                "b" => Ok(LayoutMode::LegacyB),
                other => Err(format!("invalid saved layout '{}'", other)),
            },
            SavedLayoutKind::Dynamic {
                layout_type,
                pane_count,
            } => {
                if *pane_count == 0 {
                    return Err("invalid saved layout: pane count must be at least 1".to_string());
                }

                Ok(LayoutMode::Dynamic {
                    layout_type: layout_type.clone(),
                    pane_count: *pane_count,
                })
            }
        }
    }
}

fn dynamic_split_operations(layout_type: &LayoutType, pane_count: usize) -> Vec<SplitOperation> {
    if pane_count <= 1 {
        return Vec::new();
    }

    let mut operations = Vec::with_capacity(pane_count.saturating_sub(1));
    let mut next_index = 1;

    match layout_type {
        LayoutType::Grid => build_grid_split_operations(
            &mut operations,
            &mut next_index,
            0,
            pane_count,
            SplitDirection::Horizontal,
            SplitDirection::Vertical,
        ),
        LayoutType::MainLeft => {
            let secondary_root = push_split(
                &mut operations,
                &mut next_index,
                0,
                SplitDirection::Horizontal,
            );
            build_grid_split_operations(
                &mut operations,
                &mut next_index,
                secondary_root,
                pane_count - 1,
                SplitDirection::Horizontal,
                SplitDirection::Vertical,
            );
        }
        LayoutType::MainTop => {
            let secondary_root = push_split(
                &mut operations,
                &mut next_index,
                0,
                SplitDirection::Vertical,
            );
            build_grid_split_operations(
                &mut operations,
                &mut next_index,
                secondary_root,
                pane_count - 1,
                SplitDirection::Horizontal,
                SplitDirection::Vertical,
            );
        }
    }

    operations
}

fn build_grid_split_operations(
    operations: &mut Vec<SplitOperation>,
    next_index: &mut usize,
    root_index: usize,
    pane_count: usize,
    primary_direction: SplitDirection,
    secondary_direction: SplitDirection,
) {
    if pane_count <= 1 {
        return;
    }

    let column_count = (pane_count as f64).sqrt().ceil() as usize;
    let mut column_sizes = vec![pane_count / column_count; column_count];
    for size in column_sizes.iter_mut().take(pane_count % column_count) {
        *size += 1;
    }

    let mut column_roots = vec![root_index];
    let mut current_column = root_index;

    for _ in 1..column_count {
        current_column = push_split(operations, next_index, current_column, primary_direction);
        column_roots.push(current_column);
    }

    for (column_root, column_size) in column_roots.into_iter().zip(column_sizes) {
        let mut current_cell = column_root;
        for _ in 1..column_size {
            current_cell = push_split(operations, next_index, current_cell, secondary_direction);
        }
    }
}

fn push_split(
    operations: &mut Vec<SplitOperation>,
    next_index: &mut usize,
    parent: usize,
    direction: SplitDirection,
) -> usize {
    let new_index = *next_index;
    *next_index += 1;
    operations.push(SplitOperation {
        parent,
        new_index,
        direction,
    });
    new_index
}
