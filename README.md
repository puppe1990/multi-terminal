# multi-terminal

CLI em Rust para abrir 4 painéis de terminal com agentes de IA e comandos customizáveis.

## Preview

![Preview do multi-terminal](docs/images/multi-terminal-screenshot.png)

## O que faz

- Suporta dois layouts:
  - `b` (padrão): grade 2x2
  - `a`: coluna esquerda alta + três painéis na direita
- Inicia, por padrão:
  - pane 1: shell livre
  - pane 2: `claude --dangerously-skip-permissions`
  - pane 3: `codex --yolo`
  - pane 4: `qwen --yolo`
- Permite desabilitar agentes, sobrescrever comandos por painel e definir títulos customizados.
- Permite salvar layouts nomeados e recarregá-los depois.
- Suporta o agente OpenCode como alternativa aos outros agentes de IA.

## Estratégia de execução

- No macOS:
  - usa `iTerm2` quando disponível
  - tenta instalar `iTerm2` automaticamente via Homebrew quando não estiver instalado
- Fora disso, ou se o fluxo do `iTerm2` falhar:
  - usa `tmux` quando disponível
  - cai para um fallback TUI em Rust com `portable-pty` + `crossterm`

O terminal precisa ter no mínimo `80x24`.

## Instalação

### Rodando localmente

```bash
cargo run
cargo run -- --layout a
```

### Binário compilado

```bash
./target/debug/multi-terminal
./target/debug/multi-terminal --layout a
```

### Instalação global

```bash
cargo install --path .
```

Depois:

```bash
multi-terminal
multi-terminal --layout a
```

Atalho local do repositório:

```bash
./install
```

## Uso básico

```bash
multi-terminal
multi-terminal --layout a
multi-terminal --maximize
```

## Flags principais

### Desabilitar agentes padrão

```bash
multi-terminal --no-claude
multi-terminal --no-codex --no-qwen
multi-terminal --no-opencode
```

### Usar OpenCode no painel 4

```bash
multi-terminal --pane4 "opencode --yolo" --title4 "OpenCode"
```

### Sobrescrever comandos por painel

Os painéis são indexados de `1` a `4`:

- layout `b`: `top-left`, `top-right`, `bottom-left`, `bottom-right`
- layout `a`: `left`, `right-top`, `right-bottom-left`, `right-bottom-right`

```bash
multi-terminal \
  --pane1 "lazygit" \
  --pane2 "npm run dev" \
  --pane3 "cargo test -- --nocapture" \
  --pane4 "htop"
```

### Definir títulos customizados

```bash
multi-terminal \
  --title1 "Git" \
  --title2 "App" \
  --title3 "Tests" \
  --title4 "Monitor"
```

Se `--paneN` e `--titleN` forem usados juntos, o título customizado é aplicado ao comando daquele painel.

## Layouts salvos

Salvar uma configuração:

```bash
multi-terminal \
  --layout a \
  --pane2 "npm run dev" \
  --title2 "App" \
  --maximize \
  --save team
```

Listar layouts salvos:

```bash
multi-terminal --list-layouts
```

Carregar um layout salvo:

```bash
multi-terminal --load team
```

Overrides via CLI continuam valendo ao carregar um layout salvo:

```bash
multi-terminal --load team --pane2 "lazygit" --title2 "Git" --no-qwen
```

Os layouts são persistidos no diretório de configuração do sistema em `multi-terminal/layouts.json`.

## Layouts padrão

### Layout B

```text
┌──────────────────┬──────────────────────────────┐
│ Shell            │ Claude AI                    │
├──────────────────┼──────────────────────────────┤
│ Codex            │ Qwen / OpenCode              │
└──────────────────┴──────────────────────────────┘
```

### Layout A

```text
┌──────────────┬──────────────────────────────────┐
│ Shell        │ Claude AI                        │
│              ├──────────────────────────────────┤
│              │ Codex            │ Qwen/OpenCode  │
└──────────────┴──────────────────────────────────┘
```

## Desenvolvimento

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
