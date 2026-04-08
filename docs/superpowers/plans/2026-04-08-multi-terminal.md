# multi-terminal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Criar um binário CLI em Rust (`multi-terminal`) que divide o terminal em 4 painéis com comandos pré-definidos, usando tmux quando disponível e PTY em Rust como fallback.

**Architecture:** O binário detecta tmux via `which`, e delega para o módulo tmux (que constrói e executa comandos shell) ou para o módulo pty (que usa `portable-pty` + `crossterm` para PTYs reais). O enum `Layout` centraliza a configuração dos dois layouts suportados (A e B).

**Tech Stack:** Rust, clap 4 (CLI), which 6 (detecção de tmux), portable-pty 0.8 (PTY fallback), crossterm 0.27 (renderização fallback)

---

## File Map

| Arquivo | Responsabilidade |
|---|---|
| `Cargo.toml` | Manifesto e dependências |
| `src/main.rs` | Entrypoint: parseia CLI, detecta tmux, despacha |
| `src/layout.rs` | `enum Layout`, `struct PaneConfig`, configurações dos layouts |
| `src/tmux.rs` | Constrói e executa sequência de comandos tmux |
| `src/pty.rs` | Fallback PTY: geometria, renderização, loop de eventos |
| `tests/layout_test.rs` | Testes unitários de layout e configuração |
| `tests/tmux_test.rs` | Testes de construção dos comandos tmux |

---

## Task 1: Scaffold do projeto

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Inicializar projeto cargo**

```bash
cd /Users/matheuspuppe/Desktop/Projetos/multi-terminal
cargo init --name multi-terminal
```

Expected: criado `Cargo.toml` e `src/main.rs`

- [ ] **Step 2: Substituir Cargo.toml com dependências**

Conteúdo completo de `Cargo.toml`:

```toml
[package]
name = "multi-terminal"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "multi-terminal"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
which = "6"
portable-pty = "0.8"
crossterm = "0.27"

[dev-dependencies]
```

- [ ] **Step 3: Verificar que compila**

```bash
cargo build
```

Expected: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 4: Commit**

```bash
git init
git add Cargo.toml src/main.rs
git commit -m "chore: scaffold multi-terminal project"
```

---

## Task 2: Layout module

**Files:**
- Create: `src/layout.rs`
- Create: `tests/layout_test.rs`

- [ ] **Step 1: Criar o arquivo de teste**

```bash
mkdir -p tests
```

Conteúdo de `tests/layout_test.rs`:

```rust
use multi_terminal::layout::{Layout, PaneConfig};

#[test]
fn layout_b_has_four_panes() {
    let panes = Layout::B.panes();
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_a_has_four_panes() {
    let panes = Layout::A.panes();
    assert_eq!(panes.len(), 4);
}

#[test]
fn layout_b_pane0_has_no_command() {
    let panes = Layout::B.panes();
    assert!(panes[0].command.is_none());
}

#[test]
fn layout_b_pane1_runs_claude() {
    let panes = Layout::B.panes();
    let cmd = panes[1].command.as_ref().unwrap();
    assert_eq!(cmd.program, "claude");
    assert_eq!(cmd.args, vec!["--dangerously-skip-permissions"]);
}

#[test]
fn layout_b_pane2_runs_codex() {
    let panes = Layout::B.panes();
    let cmd = panes[2].command.as_ref().unwrap();
    assert_eq!(cmd.program, "codex");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_b_pane3_runs_qwen() {
    let panes = Layout::B.panes();
    let cmd = panes[3].command.as_ref().unwrap();
    assert_eq!(cmd.program, "qwen");
    assert_eq!(cmd.args, vec!["--yolo"]);
}

#[test]
fn layout_a_pane0_is_free() {
    let panes = Layout::A.panes();
    assert!(panes[0].command.is_none());
}
```

- [ ] **Step 2: Rodar os testes para confirmar que falham**

```bash
cargo test --test layout_test 2>&1 | head -20
```

Expected: erro de compilação — `layout` não existe ainda

- [ ] **Step 3: Implementar `src/layout.rs`**

```rust
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
```

- [ ] **Step 4: Expor o módulo em `src/main.rs`**

Substitua o conteúdo de `src/main.rs` por:

```rust
pub mod layout;
pub mod tmux;
pub mod pty;

fn main() {}
```

Crie `src/tmux.rs` e `src/pty.rs` vazios (para compilar):

```rust
// src/tmux.rs
```

```rust
// src/pty.rs
```

- [ ] **Step 5: Rodar os testes e confirmar que passam**

```bash
cargo test --test layout_test
```

Expected:
```
running 7 tests
test layout_b_has_four_panes ... ok
test layout_a_has_four_panes ... ok
test layout_b_pane0_has_no_command ... ok
test layout_b_pane1_runs_claude ... ok
test layout_b_pane2_runs_codex ... ok
test layout_b_pane3_runs_qwen ... ok
test layout_a_pane0_is_free ... ok
test result: ok. 7 passed
```

- [ ] **Step 6: Commit**

```bash
git add src/layout.rs src/main.rs src/tmux.rs src/pty.rs tests/layout_test.rs
git commit -m "feat: add Layout and PaneConfig types"
```

---

## Task 3: CLI parsing com clap

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Escrever teste de CLI**

Adicione ao final de `tests/layout_test.rs`:

```rust
use multi_terminal::parse_args;

#[test]
fn default_layout_is_b() {
    let args = parse_args(&["multi-terminal"]);
    assert_eq!(args.layout, Layout::B);
}

#[test]
fn flag_layout_a_selects_layout_a() {
    let args = parse_args(&["multi-terminal", "--layout", "a"]);
    assert_eq!(args.layout, Layout::A);
}

#[test]
fn flag_layout_b_selects_layout_b() {
    let args = parse_args(&["multi-terminal", "--layout", "b"]);
    assert_eq!(args.layout, Layout::B);
}
```

- [ ] **Step 2: Rodar para confirmar que falha**

```bash
cargo test --test layout_test 2>&1 | head -10
```

Expected: erro de compilação — `parse_args` não existe

- [ ] **Step 3: Implementar CLI em `src/main.rs`**

```rust
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
```

Adicione `clap::ValueEnum` derive em `Layout` em `src/layout.rs` — não é necessário já que usamos `value_parser` customizado.

- [ ] **Step 4: Rodar os testes**

```bash
cargo test --test layout_test
```

Expected: todos os 10 testes passando

- [ ] **Step 5: Confirmar que o binário aceita os args**

```bash
cargo run -- --help
```

Expected:
```
Abre 4 painéis de terminal com agentes de IA

Usage: multi-terminal [OPTIONS]

Options:
      --layout <LAYOUT>  Layout dos painéis: a ou b (padrão: b) [default: b]
  -h, --help             Print help
```

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/layout_test.rs
git commit -m "feat: add CLI parsing with clap"
```

---

## Task 4: Módulo tmux — construção dos comandos

**Files:**
- Modify: `src/tmux.rs`
- Create: `tests/tmux_test.rs`

- [ ] **Step 1: Criar testes de construção de comandos tmux**

Conteúdo de `tests/tmux_test.rs`:

```rust
use multi_terminal::layout::Layout;
use multi_terminal::tmux::build_commands;

#[test]
fn layout_b_commands_start_with_new_session() {
    let cmds = build_commands(&Layout::B, "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_b_sends_claude_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_claude = cmds.iter().any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude, "esperado comando claude nos cmds: {:?}", cmds);
}

#[test]
fn layout_b_sends_codex_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_codex = cmds.iter().any(|c| c.contains("codex --yolo"));
    assert!(has_codex);
}

#[test]
fn layout_b_sends_qwen_command() {
    let cmds = build_commands(&Layout::B, "multi");
    let has_qwen = cmds.iter().any(|c| c.contains("qwen --yolo"));
    assert!(has_qwen);
}

#[test]
fn layout_b_ends_with_attach() {
    let cmds = build_commands(&Layout::B, "multi");
    assert!(cmds.last().unwrap().contains("attach-session"));
}

#[test]
fn layout_a_commands_start_with_new_session() {
    let cmds = build_commands(&Layout::A, "multi");
    assert!(cmds[0].starts_with("tmux new-session"));
}

#[test]
fn layout_a_sends_claude_command() {
    let cmds = build_commands(&Layout::A, "multi");
    let has_claude = cmds.iter().any(|c| c.contains("claude --dangerously-skip-permissions"));
    assert!(has_claude);
}
```

- [ ] **Step 2: Rodar para confirmar que falha**

```bash
cargo test --test tmux_test 2>&1 | head -10
```

Expected: erro de compilação — `build_commands` não existe

- [ ] **Step 3: Implementar `src/tmux.rs`**

```rust
use crate::layout::Layout;

/// Constrói a sequência de comandos tmux shell para o layout dado.
/// session_name: nome da sessão tmux a criar.
pub fn build_commands(layout: &Layout, session_name: &str) -> Vec<String> {
    let mut cmds = vec![
        // Destrói sessão existente se houver, ignora erro
        format!("tmux kill-session -t {} 2>/dev/null; true", session_name),
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
    let commands = build_commands(layout, session);

    for cmd in &commands {
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| format!("falha ao executar '{}': {}", cmd, e))?;

        // kill-session pode falhar se não existir — ignoramos (já tem `; true`)
        // attach-session assume controle do terminal — se falhar, é erro real
        if !status.success() && !cmd.contains("kill-session") {
            return Err(format!("comando tmux falhou: {}", cmd));
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Rodar os testes**

```bash
cargo test --test tmux_test
```

Expected:
```
running 7 tests
test layout_b_commands_start_with_new_session ... ok
test layout_b_sends_claude_command ... ok
test layout_b_sends_codex_command ... ok
test layout_b_sends_qwen_command ... ok
test layout_b_ends_with_attach ... ok
test layout_a_commands_start_with_new_session ... ok
test layout_a_sends_claude_command ... ok
test result: ok. 7 passed
```

- [ ] **Step 5: Commit**

```bash
git add src/tmux.rs tests/tmux_test.rs
git commit -m "feat: add tmux command builder and executor"
```

---

## Task 5: Detecção de tmux e dispatch em main

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implementar a função `run` em `src/main.rs`**

Substitua a função `run` em `src/main.rs`:

```rust
pub fn run(args: Args) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    if cols < 80 || rows < 24 {
        eprintln!(
            "Erro: terminal muito pequeno ({}x{}). Mínimo: 80x24.",
            cols, rows
        );
        std::process::exit(1);
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
```

Adicione os imports necessários no topo de `src/main.rs`:

```rust
pub mod layout;
pub mod tmux;
pub mod pty;

use clap::Parser;
use layout::Layout;
```

- [ ] **Step 2: Verificar que compila**

```bash
cargo build 2>&1
```

Expected: compila sem erros

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire tmux detection and dispatch in main"
```

---

## Task 6: Módulo PTY — geometria dos painéis

**Files:**
- Modify: `src/pty.rs`

- [ ] **Step 1: Escrever testes de geometria**

Adicione `tests/pty_test.rs`:

```rust
use multi_terminal::pty::{compute_geometry, PaneGeometry};
use multi_terminal::layout::Layout;

#[test]
fn layout_b_geometry_fills_terminal() {
    let geom = compute_geometry(&Layout::B, 100, 40);
    assert_eq!(geom.len(), 4);
    // Pane 0 e 2 devem estar na coluna esquerda
    assert_eq!(geom[0].col, 0);
    assert_eq!(geom[2].col, 0);
    // Pane 1 e 3 devem estar na coluna direita
    assert!(geom[1].col > 0);
    assert!(geom[3].col > 0);
}

#[test]
fn layout_b_geometry_pane_sizes_cover_terminal() {
    let geom = compute_geometry(&Layout::B, 100, 40);
    // largura total dos dois painéis esquerda+direita ~= total
    let left_width = geom[0].width;
    let right_width = geom[1].width;
    // +1 para a borda
    assert!(left_width + right_width + 1 <= 100);
}

#[test]
fn layout_a_geometry_left_pane_spans_full_height() {
    let geom = compute_geometry(&Layout::A, 100, 40);
    // pane 0 (esquerda) deve ter altura total
    assert_eq!(geom[0].row, 0);
    assert!(geom[0].height >= 38); // margem para bordas
}
```

- [ ] **Step 2: Rodar para confirmar que falha**

```bash
cargo test --test pty_test 2>&1 | head -10
```

Expected: erro de compilação

- [ ] **Step 3: Implementar geometria em `src/pty.rs`**

```rust
use crate::layout::Layout;

#[derive(Debug, Clone)]
pub struct PaneGeometry {
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16,
}

/// Calcula a geometria dos 4 painéis para o terminal de tamanho (cols x rows).
/// Retorna 4 geometrias na ordem: [0=livre, 1=claude, 2=codex, 3=qwen]
pub fn compute_geometry(layout: &Layout, cols: u16, rows: u16) -> Vec<PaneGeometry> {
    match layout {
        Layout::B => {
            // Layout B: 2x2 simétrico
            // Divisão: metade esquerda | metade direita, metade superior | metade inferior
            let half_col = cols / 2;
            let half_row = rows / 2;
            let right_col = half_col + 1; // +1 para borda vertical
            let right_width = cols.saturating_sub(right_col);
            let bottom_row = half_row + 1; // +1 para borda horizontal
            let bottom_height = rows.saturating_sub(bottom_row);

            vec![
                // Pane 0: topo-esq (livre)
                PaneGeometry { row: 0, col: 0, width: half_col, height: half_row },
                // Pane 1: topo-dir (claude)
                PaneGeometry { row: 0, col: right_col, width: right_width, height: half_row },
                // Pane 2: baixo-esq (codex)
                PaneGeometry { row: bottom_row, col: 0, width: half_col, height: bottom_height },
                // Pane 3: baixo-dir (qwen)
                PaneGeometry { row: bottom_row, col: right_col, width: right_width, height: bottom_height },
            ]
        }
        Layout::A => {
            // Layout A: esquerda ocupa altura total | direita dividida em cima/baixo
            // e baixo dividido em esq/dir
            let left_width = cols / 3;
            let right_col = left_width + 1;
            let right_width = cols.saturating_sub(right_col);
            let half_row = rows / 2;
            let bottom_row = half_row + 1;
            let bottom_height = rows.saturating_sub(bottom_row);
            let right_half = right_width / 2;
            let qwen_col = right_col + right_half + 1;
            let qwen_width = right_width.saturating_sub(right_half + 1);

            vec![
                // Pane 0: esq, altura total (livre)
                PaneGeometry { row: 0, col: 0, width: left_width, height: rows },
                // Pane 1: dir-topo (claude)
                PaneGeometry { row: 0, col: right_col, width: right_width, height: half_row },
                // Pane 2: dir-baixo-esq (codex)
                PaneGeometry { row: bottom_row, col: right_col, width: right_half, height: bottom_height },
                // Pane 3: dir-baixo-dir (qwen)
                PaneGeometry { row: bottom_row, col: qwen_col, width: qwen_width, height: bottom_height },
            ]
        }
    }
}

pub fn run(_layout: &Layout) -> Result<(), String> {
    // implementado no próximo task
    Err("PTY fallback ainda não implementado".to_string())
}
```

- [ ] **Step 4: Expor `compute_geometry` e `PaneGeometry` como pub**

Já estão com `pub` no código acima. Verificar que `src/main.rs` tem `pub mod pty;`.

- [ ] **Step 5: Rodar os testes**

```bash
cargo test --test pty_test
```

Expected:
```
running 3 tests
test layout_b_geometry_fills_terminal ... ok
test layout_b_geometry_pane_sizes_cover_terminal ... ok
test layout_a_geometry_left_pane_spans_full_height ... ok
test result: ok. 3 passed
```

- [ ] **Step 6: Commit**

```bash
git add src/pty.rs tests/pty_test.rs
git commit -m "feat: add PTY pane geometry calculation"
```

---

## Task 7: Módulo PTY — renderização de bordas e loop de eventos

**Files:**
- Modify: `src/pty.rs`

- [ ] **Step 1: Substituir a função `run` com implementação completa**

Substitua o conteúdo de `src/pty.rs` (mantendo `compute_geometry` e `PaneGeometry`) e adicione ao final:

```rust
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

struct Pane {
    geom: PaneGeometry,
    // Buffer de saída do processo
    output: Arc<Mutex<Vec<u8>>>,
    // Writer para enviar input ao processo
    writer: Box<dyn Write + Send>,
}

pub fn run(layout: &Layout) -> Result<(), String> {
    let (cols, rows) = terminal::size().map_err(|e| e.to_string())?;
    let geometries = compute_geometry(layout, cols, rows);
    let pane_configs = layout.panes();

    let pty_system = native_pty_system();
    let mut panes: Vec<Pane> = Vec::new();

    // Abre PTYs para painéis com comandos
    for (i, (geom, config)) in geometries.iter().zip(pane_configs.iter()).enumerate() {
        let output = Arc::new(Mutex::new(Vec::<u8>::new()));

        if let Some(cmd) = &config.command {
            let pty_size = PtySize {
                rows: geom.height,
                cols: geom.width,
                pixel_width: 0,
                pixel_height: 0,
            };

            let pair = pty_system
                .openpty(pty_size)
                .map_err(|e| format!("falha ao abrir PTY para pane {}: {}", i, e))?;

            let mut builder = CommandBuilder::new(&cmd.program);
            for arg in &cmd.args {
                builder.arg(arg);
            }

            // Spawna processo — se falhar, mostra aviso mas continua
            let _child = pair.slave.spawn_command(builder).unwrap_or_else(|e| {
                eprintln!("Aviso: não foi possível iniciar '{}': {}", cmd.program, e);
                // Retorna um child dummy — não há como continuar sem o processo
                // Neste caso, o pane ficará vazio
                panic!("spawn falhou: {}", e);
            });

            let mut reader = pair
                .master
                .try_clone_reader()
                .map_err(|e| format!("falha ao clonar reader PTY: {}", e))?;

            let output_clone = Arc::clone(&output);
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let mut lock = output_clone.lock().unwrap();
                            lock.extend_from_slice(&buf[..n]);
                            // Mantém apenas os últimos 64KB para não crescer infinitamente
                            if lock.len() > 65536 {
                                let keep = lock.len() - 65536;
                                lock.drain(..keep);
                            }
                        }
                    }
                }
            });

            let writer = pair
                .master
                .take_writer()
                .map_err(|e| format!("falha ao obter writer PTY: {}", e))?;

            panes.push(Pane { geom: geom.clone(), output, writer });
        } else {
            // Pane livre: writer descarta tudo
            struct NullWriter;
            impl Write for NullWriter {
                fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
                fn flush(&mut self) -> io::Result<()> { Ok(()) }
            }
            panes.push(Pane { geom: geom.clone(), output, writer: Box::new(NullWriter) });
        }
    }

    // Entra em modo raw e tela alternativa
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).map_err(|e| e.to_string())?;

    let mut focused: usize = 0; // pane 0 (livre) começa com foco
    let mut last_outputs: Vec<Vec<u8>> = vec![Vec::new(); panes.len()];

    loop {
        // Renderiza saída de cada pane
        for (i, pane) in panes.iter().enumerate() {
            let output = pane.output.lock().unwrap().clone();
            if output != last_outputs[i] {
                render_pane(&mut stdout, &pane.geom, &output, i == focused)
                    .map_err(|e| e.to_string())?;
                last_outputs[i] = output;
            }
        }
        stdout.flush().map_err(|e| e.to_string())?;

        // Processa eventos com timeout curto
        if event::poll(std::time::Duration::from_millis(16)).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                // Ctrl+Q: sair
                Event::Key(k) if k.code == KeyCode::Char('q') && k.modifiers == KeyModifiers::CONTROL => {
                    break;
                }
                // Ctrl+Arrows: navegar entre panes
                Event::Key(k) if k.modifiers == KeyModifiers::CONTROL => {
                    match k.code {
                        KeyCode::Right => { focused = (focused + 1) % panes.len(); }
                        KeyCode::Left => { focused = (focused + panes.len() - 1) % panes.len(); }
                        KeyCode::Down => { focused = (focused + 2) % panes.len(); }
                        KeyCode::Up => { focused = (focused + panes.len() - 2) % panes.len(); }
                        _ => {}
                    }
                }
                // Outros: envia para o pane com foco
                Event::Key(k) => {
                    let bytes = key_to_bytes(k.code);
                    if !bytes.is_empty() {
                        panes[focused].writer.write_all(&bytes).ok();
                    }
                }
                _ => {}
            }
        }
    }

    // Restaura terminal
    execute!(stdout, LeaveAlternateScreen, cursor::Show).map_err(|e| e.to_string())?;
    terminal::disable_raw_mode().map_err(|e| e.to_string())?;

    Ok(())
}

fn render_pane(
    stdout: &mut impl Write,
    geom: &PaneGeometry,
    content: &[u8],
    focused: bool,
) -> io::Result<()> {
    // Desenha borda
    let border_char = if focused { '█' } else { '│' };
    let _ = border_char; // bordas simples por ora

    // Renderiza conteúdo como texto dentro do pane
    let text = String::from_utf8_lossy(content);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(geom.height as usize);

    for (i, line) in lines[start..].iter().enumerate() {
        let row = geom.row + i as u16;
        if row >= geom.row + geom.height {
            break;
        }
        execute!(
            stdout,
            cursor::MoveTo(geom.col, row),
            Clear(ClearType::UntilNewLine),
            Print(&line[..line.len().min(geom.width as usize)])
        )?;
    }

    Ok(())
}

fn key_to_bytes(code: KeyCode) -> Vec<u8> {
    match code {
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![8],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![27],
        KeyCode::Up => vec![27, b'[', b'A'],
        KeyCode::Down => vec![27, b'[', b'B'],
        KeyCode::Right => vec![27, b'[', b'C'],
        KeyCode::Left => vec![27, b'[', b'D'],
        _ => vec![],
    }
}
```

- [ ] **Step 2: Verificar que compila**

```bash
cargo build 2>&1
```

Expected: compila sem erros (pode ter warnings de variáveis não usadas — ok)

- [ ] **Step 3: Smoke test manual básico (sem tmux)**

```bash
# Temporariamente force o modo PTY removendo tmux do PATH:
PATH="" cargo run -- --layout b 2>&1 | head -5
```

Expected: inicia sem crash (ou erro de PTY com mensagem clara)

- [ ] **Step 4: Commit**

```bash
git add src/pty.rs
git commit -m "feat: add PTY fallback with crossterm rendering and event loop"
```

---

## Task 8: Teste de integração tmux (quando disponível)

**Files:**
- Create: `tests/integration_test.rs`

- [ ] **Step 1: Criar teste de integração**

Conteúdo de `tests/integration_test.rs`:

```rust
use multi_terminal::tmux::build_commands;
use multi_terminal::layout::Layout;

#[test]
fn all_four_panes_are_configured_in_layout_b() {
    let cmds = build_commands(&Layout::B, "test-session");
    // Deve ter: kill, new-session, 4 split/select, 3 send-keys, select-pane, attach
    assert!(cmds.len() >= 9);
}

#[test]
fn all_four_panes_are_configured_in_layout_a() {
    let cmds = build_commands(&Layout::A, "test-session");
    assert!(cmds.len() >= 9);
}

#[test]
fn session_name_appears_in_all_commands() {
    let session = "my-custom-session";
    let cmds = build_commands(&Layout::B, session);
    for cmd in &cmds[1..] { // skip kill que pode não ter o nome
        assert!(cmd.contains(session), "comando sem session name: {}", cmd);
    }
}
```

- [ ] **Step 2: Rodar testes**

```bash
cargo test --test integration_test
```

Expected: todos passam

- [ ] **Step 3: Rodar todos os testes**

```bash
cargo test
```

Expected: todos os testes passam

- [ ] **Step 4: Build release**

```bash
cargo build --release
```

Expected: `target/release/multi-terminal` gerado

- [ ] **Step 5: Smoke test final com tmux (se disponível)**

```bash
which tmux && echo "tmux disponível" || echo "tmux não encontrado"
# Se disponível:
./target/release/multi-terminal
```

Expected: abre sessão tmux com 4 painéis, pane 0 livre com foco, outros rodando os comandos

- [ ] **Step 6: Commit final**

```bash
git add tests/integration_test.rs
git commit -m "feat: add integration tests and complete multi-terminal implementation"
```

---

## Resumo das dependências entre tasks

```
Task 1 (scaffold)
  └─> Task 2 (layout)
        ├─> Task 3 (CLI)
        │     └─> Task 5 (dispatch)
        ├─> Task 4 (tmux)
        │     └─> Task 5
        └─> Task 6 (geometria PTY)
              └─> Task 7 (loop PTY)
                    └─> Task 8 (integração)
```

Tasks 4 e 6 podem ser desenvolvidas em paralelo após Task 2.
