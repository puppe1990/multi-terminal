# multi-terminal — Design Spec

**Data:** 2026-04-08  
**Status:** Aprovado

---

## Visão Geral

Binário CLI em Rust chamado `multi-terminal` que, ao ser executado, divide o terminal atual em 4 painéis e inicia comandos específicos em cada um. Suporta dois layouts (B padrão, A via flag). Detecta automaticamente a presença do tmux e usa-o quando disponível; caso contrário, cai para um TUI em Rust com PTYs reais.

---

## Interface de Linha de Comando

```
multi-terminal              # abre layout B (padrão)
multi-terminal --layout a   # abre layout A
```

Dependência de parsing: `clap` com derive macros.

---

## Layouts

### Layout B (padrão)

```
┌──────────────────┬──────────────────────────────┐
│  (livre)         │  claude --dangerously-skip   │
│                  │       -permissions           │
├──────────────────┼──────────────────────────────┤
│  codex --yolo    │  qwen --yolo                 │
└──────────────────┴──────────────────────────────┘
```

- Pane 0 (topo-esq): shell livre, sem comando
- Pane 1 (topo-dir): `claude --dangerously-skip-permissions`
- Pane 2 (baixo-esq): `codex --yolo`
- Pane 3 (baixo-dir): `qwen --yolo`

### Layout A (`--layout a`)

```
┌──────────────┬──────────────────────────────────┐
│              │  claude --dangerously-skip        │
│   (livre)    ├──────────────────────────────────┤
│              │  codex --yolo  │  qwen --yolo     │
└──────────────┴──────────────────────────────────┘
```

- Pane 0 (esq, altura total): shell livre
- Pane 1 (dir-topo): `claude --dangerously-skip-permissions`
- Pane 2 (dir-baixo-esq): `codex --yolo`
- Pane 3 (dir-baixo-dir): `qwen --yolo`

---

## Arquitetura

```
src/
  main.rs      # entrypoint: parseia CLI, detecta tmux, despacha para tmux ou pty
  layout.rs    # enum Layout { A, B } + struct PaneConfig (comando, posição)
  tmux.rs      # constrói e executa sequência de comandos tmux via std::process::Command
  pty.rs       # fallback: gerencia PTYs com portable-pty + renderização crossterm
```

### Fluxo principal

1. `main` parseia args com `clap` → resolve `Layout`
2. `which::which("tmux")` → Ok = modo tmux, Err = modo PTY
3. Modo tmux: `tmux::run(layout)` constrói e executa os comandos shell
4. Modo PTY: `pty::run(layout)` inicializa terminal raw, abre PTYs e entra no loop de eventos

---

## Módulo tmux

Usa apenas `std::process::Command` — sem crates tmux externos.

**Sequência para Layout B:**
```bash
tmux new-session -d -s multi -x "$(tput cols)" -y "$(tput lines)"
tmux split-window -h -t multi:0
tmux split-window -v -t multi:0.0
tmux split-window -v -t multi:0.1
tmux send-keys -t multi:0.1 "claude --dangerously-skip-permissions" Enter
tmux send-keys -t multi:0.2 "codex --yolo" Enter
tmux send-keys -t multi:0.3 "qwen --yolo" Enter
tmux attach-session -t multi
```

**Sequência para Layout A:**
```bash
tmux new-session -d -s multi -x "$(tput cols)" -y "$(tput lines)"
tmux split-window -h -t multi:0
tmux split-window -v -t multi:0.1
tmux split-window -h -t multi:0.2
tmux send-keys -t multi:0.1 "claude --dangerously-skip-permissions" Enter
tmux send-keys -t multi:0.2 "codex --yolo" Enter
tmux send-keys -t multi:0.3 "qwen --yolo" Enter
tmux attach-session -t multi
```

Se já existir uma sessão `multi`, a existente é destruída e recriada.

---

## Módulo PTY (fallback)

Usado quando tmux não está disponível.

**Dependências:** `portable-pty`, `crossterm`

**Fluxo:**
1. `crossterm::terminal::enable_raw_mode()` + tela alternativa
2. Calcula geometria dos painéis baseada em `terminal_size()`
3. Para cada painel com comando: abre `PtyPair`, spawna processo filho com tamanho de PTY correspondente ao painel
4. Loop principal:
   - Lê stdout de cada PTY → renderiza no painel correto (com bordas crossterm)
   - Captura eventos de teclado → roteia para o painel com foco
   - Navegação: `Ctrl+←/→/↑/↓` troca painel com foco
5. Ao sair (`Ctrl+Q`): mata processos filhos, restaura terminal

---

## Tratamento de Erros

- Comando não encontrado (claude, codex, qwen): exibe aviso no painel mas não impede os outros de abrir
- tmux falha ao criar sessão: cai automaticamente para modo PTY
- Terminal muito pequeno (< 80x24): exibe erro e sai com código 1

---

## Dependências (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
which = "6"
portable-pty = "0.8"
crossterm = "0.27"
```

---

## Critérios de Sucesso

- `multi-terminal` abre 4 painéis no layout B com os comandos corretos
- `multi-terminal --layout a` abre o layout A
- Funciona com tmux instalado
- Funciona sem tmux (fallback PTY)
- Painel livre recebe foco inicial
