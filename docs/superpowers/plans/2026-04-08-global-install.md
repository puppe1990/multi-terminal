# Global Install Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Permitir instalacao global do binario `multi-terminal` e documentar o fluxo para uso com um unico comando em qualquer pasta.

**Architecture:** O projeto ja expoe um binario Cargo valido, entao a implementacao deve se apoiar no fluxo padrao `cargo install`. O trabalho aqui e tornar isso explicito e verificavel: documentacao, atalho de conveniencia e validacao real da instalacao em um prefixo temporario.

**Tech Stack:** Rust, Cargo, shell script, Markdown

---

## File Map

| Arquivo | Responsabilidade |
|---|---|
| `README.md` | Documentar instalacao global e uso |
| `install` | Atalho de conveniencia para instalar via Cargo |

### Task 1: Documentacao e atalho de instalacao

**Files:**
- Modify: `README.md`
- Create: `install`

- [ ] **Step 1: Adicionar secao de instalacao global ao README**

Incluir instrucoes para:
- `cargo install --path .`
- `cargo install --git https://github.com/puppe1990/multi-terminal`
- uso do binario global com `multi-terminal`

- [ ] **Step 2: Criar script de conveniencia**

Criar `install` executando:

```bash
#!/usr/bin/env sh
set -eu

cargo install --path .
```

- [ ] **Step 3: Validar instalacao real**

Rodar:

```bash
tmp_root="$(mktemp -d)"
CARGO_INSTALL_ROOT="$tmp_root" cargo install --path .
"$tmp_root/bin/multi-terminal" --help
rm -rf "$tmp_root"
```

Expected: instalacao concluida e help do binario exibido

- [ ] **Step 4: Rodar testes do projeto**

```bash
cargo test
```

- [ ] **Step 5: Commit**

```bash
git add README.md install docs/superpowers/plans/2026-04-08-global-install.md
git commit -m "feat: add global install flow"
```
