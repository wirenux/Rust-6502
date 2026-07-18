use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, text::{Line, Span}
};

use ratatui::widgets::{Table, Row, Block, TableState, Paragraph};
use ratatui::style::{Style, Modifier, Color};

use crossterm::{
    event::{self, Event, KeyCode::{self}}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use std::{collections::{HashMap, HashSet}, io};

use crate::cpu::CPU;
use crate::bus::Bus;
use crate::disasm::{disassemble_range, DisasmLine};

use std::{thread, time::Duration};

struct TuiState {
    memory_scroll: usize,
    stack_scroll: usize,
    running: bool,
    disasm_start: u16,
    opcode_table_state: TableState,
    disasm_lines: Vec<DisasmLine>,
    manual_selection: Option<usize>,
    total_rows: usize,
}

const TARGET_HZ: u64 = 1_000_000; // 1 MHz
const NS_PER_CYCLE: u64 = 1_000_000_000 / TARGET_HZ; // nanosecond per cycle

fn find_label_addr(lines: &[DisasmLine]) -> HashSet<u16> {
    let mut labels = HashSet::new();

    for line in lines {
        if let Some(addr_str) = line.text.split('$').nth(1) {
            let addr_str: String = addr_str.chars().take(4).collect();
            if let Ok(addr) = u16::from_str_radix(&addr_str, 16) {
                if line.text.starts_with("JSR")
                    || line.text.starts_with("JMP")
                    || line.text.starts_with('B') // BEQ, BNE, BCC, BCS, BMI, BPL, BVC, BVS
                {
                    labels.insert(addr);
                }
            }
        }
    }

    labels
}

fn build_opcode_rows<'a>(lines: &'a [DisasmLine], labels: &HashSet<u16>) -> (Vec<Row<'a>>, HashMap<u16, usize>) {
    let mut rows = Vec::new();
    let mut addr_to_row = HashMap::new();
    let mut indented = false;

    for line in lines {
        if labels.contains(&line.addr) {
            rows.push(Row::new(vec![
                Span::styled(format!("${:04X}:", line.addr), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(""),
            ]));
            indented = true;
        }

        let indent = if indented { "    " } else { "" };

        addr_to_row.insert(line.addr, rows.len());

        rows.push(Row::new(vec![
            Span::styled(format!("{}${:04X}", indent, line.addr), Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{}{}", indent, line.text)),
        ]));
    }

    (rows, addr_to_row)
}

fn flag_span(label: &str, set: bool) -> Span<'static> {
    let style = if set {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    Span::styled(format!("{} ", label), style)
}

fn render_flags(frame: &mut Frame, area: Rect, cpu: &CPU, state: &TuiState) {
    let status_span = if state.running {
        Span::styled("RUN", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("HALT", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    };

    let flag_spans_width = 6 * 2;
    let status_width = status_span.content.len();
    let inner_width = area.width.saturating_sub(2) as usize;
    let padding = inner_width.saturating_sub(flag_spans_width + status_width);

    let mut spans = vec![
        flag_span("N", cpu.get_flag(CPU::NEGATIVE_FLAG)),
        flag_span("V", cpu.get_flag(CPU::OVERFLOW_FLAG)),
        flag_span("D", cpu.get_flag(CPU::DECIMAL_FLAG)),
        flag_span("I", cpu.get_flag(CPU::INTERRUPT_FLAG)),
        flag_span("Z", cpu.get_flag(CPU::ZERO_FLAG)),
        flag_span("C", cpu.get_flag(CPU::CARRY_FLAG)),
    ];

    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(status_span);

    let flags_line = Line::from(spans);

    let flag_widget = Paragraph::new(flags_line)
        .block(Block::bordered().title("Flags"));

    frame.render_widget(flag_widget, area);
}

fn render_register(frame: &mut Frame, area: Rect, cpu: &CPU) {
    let header = Row::new(vec!["AC", "XR", "YR", "SP"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let values_row = Row::new(vec![
        format!("{:02X}", cpu.reg_a),
        format!("{:02X}", cpu.reg_x),
        format!("{:02X}", cpu.reg_y),
        format!("{:02X}", cpu.sp),
    ]);

    let register_table = Table::new(
        vec![values_row],
        [
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ],
    )
        .column_spacing(1)
        .header(header)
        .block(Block::bordered().title("Register"));

    frame.render_widget(register_table, area);
}

pub fn render(frame: &mut Frame, cpu: &mut CPU, bus: &mut Bus, state: &mut TuiState) {
    let outer_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // left: register/todo
            Constraint::Percentage(80), // rest: opcodes + flags/memory/stack
        ])
        .split(frame.area());

    let left_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(outer_chunk[0]);

    let main_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(outer_chunk[1]);

    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunk[1]);

    let header = Row::new(vec!["ADDR", "INSTRUCTION"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let labels = find_label_addr(&state.disasm_lines);
    let (rows, addr_to_row) = build_opcode_rows(&state.disasm_lines, &labels);
    state.total_rows = rows.len();

    let selected_index = match state.manual_selection {
        Some(idx) => Some(idx),
        None => addr_to_row.get(&cpu.pc).copied(),
    };
    state.opcode_table_state.select(selected_index);

    let visible_height = main_chunk[0].height.saturating_sub(3) as usize;
    let max_offset = rows.len().saturating_sub(visible_height);

    if state.manual_selection.is_none() {
        if let Some(idx) = selected_index {
            let desired_offset = idx.saturating_sub(5);
            *state.opcode_table_state.offset_mut() = desired_offset.min(max_offset);
        }
    }

    let opcode_table = Table::new(rows, [
        Constraint::Length(9),
        Constraint::Min(10),
    ])
        .column_spacing(1)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .block(Block::bordered().title("Opcodes"));

    frame.render_stateful_widget(opcode_table, main_chunk[0], &mut state.opcode_table_state);

    render_register(frame, left_chunk[1], cpu);
    render_flags(frame, left_chunk[0], cpu, state);
    frame.render_widget(Block::bordered().title("Memory"), right_chunk[1]);
    frame.render_widget(Block::bordered().title("Stack"), right_chunk[2]);
}

pub fn run(cpu: &mut CPU, bus: &mut Bus, disasm_start: u16) -> io::Result<()> {
    let _ = enable_raw_mode();

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let disasm_lines = disassemble_range(bus, disasm_start, 2000); // make cache

    let mut state = TuiState {
        memory_scroll: 0,
        stack_scroll: 0,
        running: false,
        disasm_start,
        opcode_table_state: TableState::default(),
        disasm_lines,
        manual_selection: None,
        total_rows: 0,
    };

    loop {
        terminal.draw(|frame| render(frame, cpu, bus, &mut state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('n') => {
                        cpu.clock_tick(bus);
                        state.manual_selection = None;
                    },
                    KeyCode::Char('r') => {
                        state.running = !state.running;
                        state.manual_selection = None;
                    },
                    KeyCode::Up => {
                        let current = state.manual_selection.unwrap_or_else(|| {
                            state.opcode_table_state.selected().unwrap_or(0)
                        });
                        state.manual_selection = Some(current.saturating_sub(1));
                    }
                    KeyCode::Down => {
                        let current = state.manual_selection.unwrap_or_else(|| {
                            state.opcode_table_state.selected().unwrap_or(0)
                        });
                        let max = state.total_rows.saturating_sub(1);
                        state.manual_selection = Some((current + 1).min(max));
                    },
                    _ => {}
                }
            }
        }

        if state.running && !cpu.halted {
            let delay_ns = NS_PER_CYCLE * cpu.last_cycles as u64;
            thread::sleep(Duration::from_nanos(delay_ns));
            cpu.clock_tick(bus);
            state.manual_selection = None;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = terminal.show_cursor();

    Ok(())
}