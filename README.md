# multi-terminal

CLI em Rust para abrir 4 painéis no terminal com agentes pré-configurados.

## O que faz

- No macOS, prefere abrir uma nova janela com 4 abas separadas:
  - `iTerm2` quando instalado
  - `Terminal.app` como fallback
- Fora disso, usa `tmux` quando disponível para montar os painéis no terminal atual.
- Se nada disso estiver disponível, cai para um fallback em Rust com `portable-pty` + `crossterm`.
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

## Modos de Execucao

### macOS com abas automaticas

Se `iTerm2` ou `Terminal.app` estiver instalado, o comando abre uma nova janela com 4 abas no mesmo diretorio atual e executa automaticamente:

- Aba 1: shell livre
- Aba 2: `claude --dangerously-skip-permissions`
- Aba 3: `codex --yolo`
- Aba 4: `qwen --yolo`

Ordem de preferencia:

1. `iTerm2`
2. `Terminal.app`
3. `tmux`
4. fallback PTY

Para a experiencia mais proxima do que voce descreveu, use macOS com `iTerm2` instalado.

## Instalacao Global

Para instalar o binario globalmente na sua maquina:

```bash
cargo install --path .
```

Depois disso, em qualquer pasta:

```bash
multi-terminal
multi-terminal --layout a
```

Se preferir instalar direto do GitHub:

```bash
cargo install --git https://github.com/puppe1990/multi-terminal
```

Tambem existe um atalho local no repo:

```bash
./install
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
