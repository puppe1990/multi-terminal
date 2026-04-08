# multi-terminal

CLI em Rust para abrir 4 painéis no terminal com agentes pré-configurados.

## O que faz

- Usa `tmux` quando disponível para montar os painéis no terminal atual.
- Cai para um fallback em Rust com `portable-pty` + `crossterm` quando `tmux` não está instalado.
- Suporta dois layouts:
  - `b` (padrão)
  - `a`

## Uso

```bash
cargo run
cargo run -- --layout a
```

Ou, após compilar:

```bash
./target/debug/multi-terminal
./target/debug/multi-terminal --layout a
```

## Comandos por painel

### Layout B

- Pane 0: shell livre
- Pane 1: `claude --dangerously-skip-permissions`
- Pane 2: `codex --yolo`
- Pane 3: `qwen --yolo`

### Layout A

- Pane 0: shell livre
- Pane 1: `claude --dangerously-skip-permissions`
- Pane 2: `codex --yolo`
- Pane 3: `qwen --yolo`

## Desenvolvimento

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
