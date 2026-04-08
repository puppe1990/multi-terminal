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
            AgentType::OpenCode => Some(Command::new("opencode", &["--yolo"])),
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
        
        agents
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    pub layout: String,
    pub agents: Vec<AgentConfig>,
    pub maximize: bool,
}

impl SavedLayout {
    pub fn config_path() -> std::path::PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("multi-terminal");
        path.push("layouts.json");
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
        // Try to parse as legacy first
        match self.layout.to_lowercase().as_str() {
            "a" => Ok(LayoutMode::LegacyA),
            "b" => Ok(LayoutMode::LegacyB),
            _ => {
                // This will be updated in Task 3 when we add dynamic persistence
                Err(format!("invalid saved layout '{}'", self.layout))
            }
        }
    }
}
