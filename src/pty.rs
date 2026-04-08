use crate::layout::Layout;
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
            let half_col = cols / 2;
            let half_row = rows / 2;
            let right_col = half_col + 1;
            let right_width = cols.saturating_sub(right_col);
            let bottom_row = half_row + 1;
            let bottom_height = rows.saturating_sub(bottom_row);

            vec![
                PaneGeometry { row: 0, col: 0, width: half_col, height: half_row },
                PaneGeometry { row: 0, col: right_col, width: right_width, height: half_row },
                PaneGeometry { row: bottom_row, col: 0, width: half_col, height: bottom_height },
                PaneGeometry { row: bottom_row, col: right_col, width: right_width, height: bottom_height },
            ]
        }
        Layout::A => {
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
                PaneGeometry { row: 0, col: 0, width: left_width, height: rows },
                PaneGeometry { row: 0, col: right_col, width: right_width, height: half_row },
                PaneGeometry { row: bottom_row, col: right_col, width: right_half, height: bottom_height },
                PaneGeometry { row: bottom_row, col: qwen_col, width: qwen_width, height: bottom_height },
            ]
        }
    }
}

struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

struct Pane {
    geom: PaneGeometry,
    output: Arc<Mutex<Vec<u8>>>,
    writer: Box<dyn Write + Send>,
}

pub fn run(layout: &Layout) -> Result<(), String> {
    let (cols, rows) = terminal::size().map_err(|e| e.to_string())?;
    let geometries = compute_geometry(layout, cols, rows);
    let pane_configs = layout.panes();

    let pty_system = native_pty_system();
    let mut panes: Vec<Pane> = Vec::new();

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

            let child_result = pair.slave.spawn_command(builder);
            if let Err(e) = child_result {
                eprintln!("Aviso: não foi possível iniciar '{}': {}", cmd.program, e);
                // Pane fica vazio com NullWriter
                panes.push(Pane { geom: geom.clone(), output, writer: Box::new(NullWriter) });
                continue;
            }

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
                            // Mantém apenas os últimos 64KB
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
            panes.push(Pane { geom: geom.clone(), output, writer: Box::new(NullWriter) });
        }
    }

    // Entra em modo raw e tela alternativa
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).map_err(|e| e.to_string())?;
    let _guard = TerminalGuard;

    let mut focused: usize = 0;
    let mut last_outputs: Vec<Vec<u8>> = vec![Vec::new(); panes.len()];

    loop {
        for (i, pane) in panes.iter().enumerate() {
            let output = pane.output.lock().unwrap().clone();
            if output != last_outputs[i] {
                render_pane(&mut stdout, &pane.geom, &output, i == focused)
                    .map_err(|e| e.to_string())?;
                last_outputs[i] = output;
            }
        }
        stdout.flush().map_err(|e| e.to_string())?;

        if event::poll(std::time::Duration::from_millis(16)).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                Event::Key(k) if k.code == KeyCode::Char('q') && k.modifiers == KeyModifiers::CONTROL => {
                    break;
                }
                Event::Key(k) if k.modifiers == KeyModifiers::CONTROL => {
                    match k.code {
                        KeyCode::Right => { focused = (focused + 1) % panes.len(); }
                        KeyCode::Left => { focused = (focused + panes.len() - 1) % panes.len(); }
                        KeyCode::Down => { focused = (focused + 2) % panes.len(); }
                        KeyCode::Up => { focused = (focused + panes.len() - 2) % panes.len(); }
                        _ => {}
                    }
                }
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

    Ok(())
}

fn render_pane(
    stdout: &mut impl Write,
    geom: &PaneGeometry,
    content: &[u8],
    _focused: bool,
) -> io::Result<()> {
    let text = String::from_utf8_lossy(content);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(geom.height as usize);

    for (i, line) in lines[start..].iter().enumerate() {
        let row = geom.row + i as u16;
        if row >= geom.row + geom.height {
            break;
        }
        let truncated = if line.len() > geom.width as usize {
            &line[..geom.width as usize]
        } else {
            line
        };
        execute!(
            stdout,
            cursor::MoveTo(geom.col, row),
            Clear(ClearType::UntilNewLine),
            Print(truncated)
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
