# Variable Panes Design Spec

**Data:** 2026-04-08
**Status:** Aprovado em conversa

## Objetivo

Permitir que o `multi-terminal` abra uma quantidade variável de panes em vez de assumir sempre 4 panes fixos. O modelo novo deve ser orientado por tipo de layout e quantidade de panes, com overrides de comando e título por índice.

## Escopo

Esta mudança cobre:

1. Novo modelo de CLI baseado em `--layout-type` e `--panes`
2. Suporte a layouts dinâmicos `grid`, `main-left` e `main-top`
3. Overrides de panes por índice para comandos e títulos
4. Persistência de layouts dinâmicos salvos
5. Compatibilidade com o modelo legado `--layout a|b`
6. Atualização de `tmux`, `iTerm2`, `PTY` e documentação

Esta mudança não cobre:

1. Editor interativo de panes
2. Redimensionamento manual por proporção via CLI
3. Layouts arbitrários definidos por árvore de splits
4. Remoção imediata dos aliases legados `a` e `b`

## Interface de CLI

### Novo caminho principal

O caminho principal da CLI passa a ser:

```bash
multi-terminal --layout-type grid --panes 6
multi-terminal --layout-type main-left --panes 5
multi-terminal --layout-type main-top --panes 7
```

Novas flags:

- `--layout-type <grid|main-left|main-top>`
- `--panes <N>`
- `--pane <INDEX=COMMAND>` repetível
- `--title <INDEX=TITLE>` repetível

Exemplos:

```bash
multi-terminal --layout-type grid --panes 6 --pane 2="claude --dangerously-skip-permissions"
multi-terminal --layout-type main-left --panes 5 --title 1=Shell --title 4=Tests
```

### Compatibilidade legada

O caminho legado continua válido:

```bash
multi-terminal --layout a
multi-terminal --layout b
```

Regras:

- `--layout a|b` continua significando layout fixo com 4 panes
- `--layout` é mutuamente exclusivo com `--layout-type`
- `--layout` é mutuamente exclusivo com `--panes`
- layouts legados continuam aceitando overrides por índice compatíveis com 4 panes

## Modelo de dados

### Layout resolvido

O modelo interno deixa de depender apenas de `Layout::A | Layout::B`.

Estrutura alvo:

- layout legado fixo:
  - `LegacyA`
  - `LegacyB`
- layout dinâmico:
  - `Dynamic { layout_type, pane_count }`

### Tipo de layout dinâmico

Novo enum:

- `Grid`
- `MainLeft`
- `MainTop`

### Agentes padrão

A lista padrão de agentes passa a escalar com a quantidade de panes:

1. pane 1: shell livre
2. pane 2: `claude --dangerously-skip-permissions`
3. pane 3: `codex --yolo`
4. pane 4: `qwen --yolo`
5. pane 5+: shell livre

O número de agentes resolvidos deve sempre coincidir com o total de panes.

## Semântica dos overrides

### Comandos

`--pane <INDEX=COMMAND>` sobrescreve o comando do pane informado.

Regras:

- índices são baseados em 1
- o índice deve existir no layout resolvido
- índice fora do intervalo retorna erro claro
- um comando customizado usa o valor informado também como título padrão, a menos que `--title` para o mesmo índice seja informado

### Títulos

`--title <INDEX=TITLE>` sobrescreve apenas o título do pane informado.

Regras:

- índices são baseados em 1
- o índice deve existir no layout resolvido
- índice fora do intervalo retorna erro claro

## Geometria dos layouts

### Grid

Distribui os panes em uma grade automática, buscando uma divisão visualmente equilibrada.

Regras:

- a grade deve ser o mais quadrada possível
- para `N=1`, existe apenas um pane
- para `N>1`, a quantidade de colunas e linhas é derivada de `N`
- panes da última linha podem sobrar parcialmente quando `N` não fecha grade perfeita

### MainLeft

O pane 1 ocupa uma área principal à esquerda. Os demais panes são distribuídos na área da direita.

Regras:

- para `N=1`, existe apenas um pane
- para `N>1`, o pane 1 ocupa a coluna principal à esquerda
- panes 2..N usam uma subgrade automática na área da direita

### MainTop

O pane 1 ocupa uma área principal no topo. Os demais panes são distribuídos na área inferior.

Regras:

- para `N=1`, existe apenas um pane
- para `N>1`, o pane 1 ocupa a faixa principal do topo
- panes 2..N usam uma subgrade automática na área de baixo

## Backends

### tmux

O backend `tmux` deixa de usar um `match` fixo por layout de 4 panes e passa a consumir um plano de splits derivado da geometria resolvida.

Objetivo:

- suportar quantidade variável de panes
- preservar o envio de comandos por índice de pane
- continuar anexando à sessão ao final

### iTerm2

O backend `iTerm2` também deixa de depender de AppleScript fixo para 4 panes.

Objetivo:

- gerar splits a partir do layout resolvido
- nomear panes por índice resolvido
- executar `cd` + comando em cada pane

### PTY

O backend `PTY` passa a calcular geometria variável e criar a quantidade correspondente de PTYs.

Objetivo:

- suportar todos os panes resolvidos
- preservar foco e renderização
- evitar assumir navegação fixa baseada em 4 panes

### Terminal.app

`Terminal.app` permanece como fallback simples e não tenta reproduzir fielmente os splits dinâmicos. Quando usado, pode continuar abrindo sessões em abas ou janelas separadas com os comandos correspondentes.

## Persistência

Layouts salvos precisam suportar o modelo novo sem quebrar leitura do formato antigo.

### Leitura

Deve continuar aceitando layouts antigos com:

- `layout: "a"` ou `layout: "b"`
- `agents` com 4 entradas

### Escrita

Layouts novos devem persistir:

- o tipo de layout resolvido
- a quantidade de panes
- a lista de agentes
- o estado de maximização

O formato novo deve ser explícito o suficiente para não depender de inferência baseada apenas no tamanho do vetor de agentes.

## Validação e erros

Regras mínimas:

- `--panes` aceita mínimo `1`
- máximo inicial recomendado: `9`
- `--layout-type` sem `--panes` usa um default explícito definido na implementação
- `--pane` e `--title` com índice inválido retornam erro com o índice informado e o total de panes
- combinações mutuamente exclusivas de flags retornam erro de parsing

## Testes

Cobertura mínima esperada:

1. parsing de `--layout-type`, `--panes`, `--pane`, `--title`
2. compatibilidade de `--layout a|b`
3. resolução de agentes padrão para contagens variáveis
4. validação de índices inválidos
5. geometria para `grid`, `main-left`, `main-top`
6. geração de comandos `tmux` para contagens variáveis
7. leitura e gravação de layouts no formato legado e novo

## Documentação

`README.md` deve ser atualizado para refletir:

- o caminho principal com `--layout-type` e `--panes`
- os tipos de layout suportados
- o novo formato de overrides por índice
- a compatibilidade com `--layout a|b`

## Critérios de sucesso

- o usuário consegue abrir 1 a 9 panes com `grid`, `main-left` e `main-top`
- os panes extras recebem comportamento padrão previsível
- overrides por índice funcionam em layouts dinâmicos e legados
- `tmux`, `iTerm2` e `PTY` deixam de depender de 4 panes fixos
- layouts salvos antigos continuam carregando
- layouts novos são persistidos com o modelo expandido
