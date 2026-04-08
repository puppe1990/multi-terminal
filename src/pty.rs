use crate::layout::{AgentConfig, Command as PaneCommand, Layout};
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

/// Calculates geometry for 4 panes given terminal size (cols x rows).
/// Returns 4 geometries in pane order.
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
                PaneGeometry {
                    row: 0,
                    col: 0,
                    width: half_col,
                    height: half_row,
                },
                PaneGeometry {
                    row: 0,
                    col: right_col,
                    width: right_width,
                    height: half_row,
                },
                PaneGeometry {
                    row: bottom_row,
                    col: 0,
                    width: half_col,
                    height: bottom_height,
                },
                PaneGeometry {
                    row: bottom_row,
                    col: right_col,
                    width: right_width,
                    height: bottom_height,
                },
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
                PaneGeometry {
                    row: 0,
                    col: 0,
                    width: left_width,
                    height: rows,
                },
                PaneGeometry {
                    row: 0,
                    col: right_col,
                    width: right_width,
                    height: half_row,
                },
                PaneGeometry {
                    row: bottom_row,
                    col: right_col,
                    width: right_half,
                    height: bottom_height,
                },
                PaneGeometry {
                    row: bottom_row,
                    col: qwen_col,
                    width: qwen_width,
                    height: bottom_height,
                },
            ]
        }
    }
}

pub fn command_for_pane(config: &AgentConfig) -> PaneCommand {
    match &config.agent_type {
        crate::layout::AgentType::Custom(command) if config.command.is_none() => {
            shell_command_for(command)
        }
        _ => config
            .effective_command()
            .unwrap_or_else(default_shell_command),
    }
}

fn default_shell_command() -> PaneCommand {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    PaneCommand {
        program: shell,
        args: Vec::new(),
    }
}

fn shell_command_for(command: &str) -> PaneCommand {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    PaneCommand {
        program: shell,
        args: vec!["-lc".to_string(), command.to_string()],
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
    title: String,
    output: Arc<Mutex<Vec<u8>>>,
    writer: Box<dyn Write + Send>,
}

pub fn run(layout: &Layout, agents: &[AgentConfig]) -> Result<(), String> {
    let (cols, rows) = terminal::size().map_err(|e| e.to_string())?;
    let geometries = compute_geometry(layout, cols, rows);
    let pane_configs = layout.panes(agents);

    let pty_system = native_pty_system();
    let mut panes: Vec<Pane> = Vec::new();

    for (i, (geom, config)) in geometries.iter().zip(pane_configs.iter()).enumerate() {
        let output = Arc::new(Mutex::new(Vec::<u8>::new()));

        let cmd = command_for_pane(config);
        let pty_size = PtySize {
            rows: geom.height,
            cols: geom.width,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| format!("failed to open PTY for pane {}: {}", i, e))?;

        let mut builder = CommandBuilder::new(&cmd.program);
        for arg in &cmd.args {
            builder.arg(arg);
        }

        let child_result = pair.slave.spawn_command(builder);
        if let Err(e) = child_result {
            {
                let mut lock = output.lock().unwrap();
                lock.extend_from_slice(
                    format!("Warning: could not start '{}': {}\n", cmd.program, e).as_bytes(),
                );
            }

            panes.push(Pane {
                geom: geom.clone(),
                title: config.effective_title(),
                output,
                writer: Box::new(NullWriter),
            });
            continue;
        }

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("failed to clone PTY reader: {}", e))?;

        let output_clone = Arc::clone(&output);
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut lock = output_clone.lock().unwrap();
                        lock.extend_from_slice(&buf[..n]);
                        // Keep only last 64KB
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
            .map_err(|e| format!("failed to get PTY writer: {}", e))?;

        panes.push(Pane {
            geom: geom.clone(),
            title: config.effective_title(),
            output,
            writer,
        });
    }

    // Enter raw mode and alternate screen
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).map_err(|e| e.to_string())?;
    let _guard = TerminalGuard;

    let mut focused: usize = 0;
    let mut last_outputs: Vec<Option<Vec<u8>>> = vec![None; panes.len()];

    loop {
        for (i, pane) in panes.iter().enumerate() {
            let output = pane.output.lock().unwrap().clone();
            if last_outputs[i].as_ref() != Some(&output) {
                render_pane(&mut stdout, &pane.geom, &pane.title, &output, i == focused)
                    .map_err(|e| e.to_string())?;
                last_outputs[i] = Some(output);
            }
        }
        stdout.flush().map_err(|e| e.to_string())?;

        if event::poll(std::time::Duration::from_millis(16)).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                Event::Key(k)
                    if k.code == KeyCode::Char('q') && k.modifiers == KeyModifiers::CONTROL =>
                {
                    break;
                }
                Event::Key(k) if k.modifiers == KeyModifiers::CONTROL => match k.code {
                    KeyCode::Right => {
                        focused = (focused + 1) % panes.len();
                        invalidate_all_panes(&mut last_outputs);
                    }
                    KeyCode::Left => {
                        focused = (focused + panes.len() - 1) % panes.len();
                        invalidate_all_panes(&mut last_outputs);
                    }
                    KeyCode::Down => {
                        focused = (focused + 2) % panes.len();
                        invalidate_all_panes(&mut last_outputs);
                    }
                    KeyCode::Up => {
                        focused = (focused + panes.len() - 2) % panes.len();
                        invalidate_all_panes(&mut last_outputs);
                    }
                    _ => {}
                },
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
    title: &str,
    content: &[u8],
    focused: bool,
) -> io::Result<()> {
    for (i, line) in render_lines(geom, title, content, focused)
        .into_iter()
        .enumerate()
    {
        execute!(
            stdout,
            cursor::MoveTo(geom.col, geom.row + i as u16),
            Clear(ClearType::UntilNewLine),
            Print(line)
        )?;
    }

    Ok(())
}

pub fn render_lines(
    geom: &PaneGeometry,
    title: &str,
    content: &[u8],
    focused: bool,
) -> Vec<String> {
    if geom.width == 0 || geom.height == 0 {
        return Vec::new();
    }

    if geom.width < 2 || geom.height < 2 {
        return vec![" ".repeat(geom.width as usize); geom.height as usize];
    }

    let inner_width = (geom.width - 2) as usize;
    let inner_height = (geom.height - 2) as usize;
    let horizontal = if focused { '=' } else { '-' };
    let border = border_line(inner_width, title, horizontal);

    let text = normalize_terminal_output(content);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(inner_height);

    let mut rendered = Vec::with_capacity(geom.height as usize);
    rendered.push(border.clone());

    for line in lines[start..].iter().take(inner_height) {
        let mut visible = line.chars().take(inner_width).collect::<String>();
        let padding = inner_width.saturating_sub(visible.chars().count());
        visible.push_str(&" ".repeat(padding));
        rendered.push(format!("|{}|", visible));
    }

    while rendered.len() < geom.height as usize - 1 {
        rendered.push(format!("|{}|", " ".repeat(inner_width)));
    }

    rendered.push(border);
    rendered
}

fn border_line(inner_width: usize, title: &str, horizontal: char) -> String {
    if inner_width == 0 {
        return "++".to_string();
    }

    let clean_title = title.trim();
    if clean_title.is_empty() || clean_title.len() + 2 >= inner_width {
        return format!("+{}+", horizontal.to_string().repeat(inner_width));
    }

    let label = format!(" {} ", clean_title);
    let left = (inner_width - label.len()) / 2;
    let right = inner_width - label.len() - left;

    format!(
        "+{}{}{}+",
        horizontal.to_string().repeat(left),
        label,
        horizontal.to_string().repeat(right)
    )
}

pub fn normalize_terminal_output(content: &[u8]) -> String {
    let text = String::from_utf8_lossy(content);
    let mut normalized = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\u{1b}' => match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if next == '\u{7}' || (prev == '\u{1b}' && next == '\\') {
                            break;
                        }
                        prev = next;
                    }
                }
                _ => {}
            },
            '\r' => {}
            '\n' | '\t' => normalized.push(ch),
            ch if ch.is_control() => {}
            _ => normalized.push(ch),
        }
    }

    normalized
}

pub fn invalidate_all_panes(last_outputs: &mut [Option<Vec<u8>>]) {
    for entry in last_outputs.iter_mut() {
        *entry = None;
    }
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
