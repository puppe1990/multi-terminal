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
    Err("PTY fallback ainda não implementado".to_string())
}
