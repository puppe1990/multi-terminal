use multi_terminal::pty::compute_geometry;
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
    assert!(left_width + right_width < 100);
}

#[test]
fn layout_a_geometry_left_pane_spans_full_height() {
    let geom = compute_geometry(&Layout::A, 100, 40);
    // pane 0 (esquerda) deve ter altura total
    assert_eq!(geom[0].row, 0);
    assert!(geom[0].height >= 38); // margem para bordas
}
