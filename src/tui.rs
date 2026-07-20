use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, text::{Line, Span}, widgets::ScrollbarState
};

use ratatui::widgets::{Table, Row, Block, TableState, Paragraph, Scrollbar, ScrollbarOrientation};
use ratatui::style::{Style, Modifier, Color};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::{self}, KeyCode::{self}, MouseEventKind}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use std::{collections::{HashMap, HashSet}, io};

use crate::cpu::CPU;
use crate::bus::Bus;
use crate::disasm::{disassemble_range, DisasmLine};

use std::{thread, time::Duration};

struct TuiState {
    running: bool,
    disasm_lines: Vec<DisasmLine>,
    manual_selection: Option<usize>,
    total_rows: usize,
    memory_area: Rect,
    memory_table_state: TableState,
    memory_scroll_row: usize,
    stack_area: Rect,
    stack_table_state: TableState,
    stack_manual_scroll: Option<usize>,
    opcode_area: Rect,
    opcode_table_state: TableState,
    instructions_per_second: u32,
}

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
        format!("$01{:02X}", cpu.sp),
    ]);

    let register_table = Table::new(
        vec![values_row],
        [
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
        ],
    )
        .column_spacing(1)
        .header(header)
        .block(Block::bordered().title("Register"));

    frame.render_widget(register_table, area);
}

fn render_stack(frame: &mut Frame, area: Rect, cpu: &CPU, bus: &Bus, state: &mut TuiState) {
    let header = Row::new(vec!["ADDR", "VALUE"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = (0x00..=0xFF).rev().map(|offset: u8| {
        let addr = 0x0100u16 + offset as u16;
        let value = bus.read_ram(addr);
        let is_top = offset == cpu.sp;
        let style = if is_top {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Row::new(vec![
            Span::styled(format!("${:04X}", addr), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:02X}", value), style),
        ])
    }).collect();

    let sp_row_index = (0xFFu16 - cpu.sp as u16) as usize;

    if state.stack_manual_scroll.is_none() {
        state.stack_table_state.select(Some(sp_row_index));
    } else {
        state.stack_table_state.select(None);
        *state.stack_table_state.offset_mut() = state.stack_manual_scroll.unwrap();
    }

    let stack_table = Table::new(rows, [Constraint::Length(7), Constraint::Length(5)])
        .column_spacing(1)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .block(Block::bordered().title("Stack"));

    frame.render_stateful_widget(stack_table, area, &mut state.stack_table_state);

    let visible_height = area.height.saturating_sub(3) as usize;
    let max_offset = 256usize.saturating_sub(visible_height);

    let mut scrollbar_state = ScrollbarState::new(max_offset).position(state.stack_table_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn build_memory_rows(bus: &Bus, start_row: usize, visible_count: usize) -> Vec<Row<'static>> {
    (start_row..(start_row + visible_count).min(4096)).map(|row_idx| {
        let addr = (row_idx * 16) as u16;
        let mut hex_bytes = String::new();

        for col in 0..16 {
            hex_bytes.push_str(&format!("{:02X} ", bus.read_ram(addr.wrapping_add(col))));
        }

        Row::new(vec![
            Span::styled(format!("${:04X}", addr), Style::default().fg(Color::DarkGray)),
            Span::raw(hex_bytes),
        ])
    }).collect()
}


fn render_memory(frame: &mut Frame, area: Rect, bus: &Bus, state: &mut TuiState) {
    let visible_count = area.height.saturating_sub(3) as usize;
    let start_row = state.memory_scroll_row;

    let rows = build_memory_rows(bus, start_row, visible_count);

    *state.memory_table_state.offset_mut() = 0;

    let column_labels: String = (0..16).map(|col| format!("{:02X} ", col)).collect();

    let header = Row::new(vec![
        Span::raw(""),
        Span::styled(column_labels, Style::default().add_modifier(Modifier::BOLD)),
    ])
        .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::DarkGray));

    let memory_table = Table::new(rows, [
        Constraint::Length(7),
        Constraint::Min(10),
    ])
        .column_spacing(1)
        .header(header)
        .block(Block::bordered().title("Memory"));

    frame.render_stateful_widget(memory_table, area, &mut state.memory_table_state);

    let max_offset = 4096usize.saturating_sub(visible_count);

    let mut scrollbar_state = ScrollbarState::new(max_offset)
        .position(state.memory_scroll_row);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);

}

fn render_opcodes(frame: &mut Frame, area: Rect, cpu: &mut CPU, state: &mut TuiState) {
    let visible_height = area.height.saturating_sub(3) as usize;

    let header = Row::new(vec!["ADDR", "INSTRUCTION"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let labels = find_label_addr(&state.disasm_lines);

    let (rows, addr_to_row) = build_opcode_rows(&state.disasm_lines, &labels);
    state.total_rows = rows.len();
    let max_offset = state.total_rows.saturating_sub(visible_height);

    let selected_index = match state.manual_selection {
        Some(idx) => Some(idx),
        None => addr_to_row.get(&cpu.pc).copied(),
    };
    state.opcode_table_state.select(selected_index);

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

    frame.render_stateful_widget(opcode_table, area, &mut state.opcode_table_state);

    let mut scrollbar_state = ScrollbarState::new(max_offset)
        .position(state.opcode_table_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn render(frame: &mut Frame, cpu: &mut CPU, state: &mut TuiState, bus: &mut Bus) {
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
            Constraint::Percentage(100),
        ])
        .split(outer_chunk[0]);

    let main_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(outer_chunk[1]);

    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunk[1]);

    render_flags(frame, left_chunk[0], cpu, state);
    render_memory(frame, right_chunk[0], bus, state);
    render_opcodes(frame, main_chunk[0], cpu, state);
    render_register(frame, left_chunk[1], cpu);
    render_stack(frame, left_chunk[2], cpu, bus, state);

    state.memory_area = right_chunk[0];
    state.stack_area = left_chunk[2];
    state.opcode_area = main_chunk[0];
}

pub fn run(cpu: &mut CPU, bus: &mut Bus, disasm_start: u16) -> io::Result<()> {
    let _ = enable_raw_mode();

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let disasm_lines = disassemble_range(bus, disasm_start, 2000); // make cache

    let mut state = TuiState {
        running: false,
        disasm_lines,
        manual_selection: None,
        total_rows: 0,
        stack_table_state: TableState::default(),
        memory_scroll_row: 0,
        memory_table_state: TableState::default(),
        stack_manual_scroll: None,
        memory_area: Rect::default(),
        stack_area: Rect::default(),
        opcode_area: Rect::default(),
        opcode_table_state: TableState::default(),
        instructions_per_second: 100,
    };

    let mut should_quit = false;

    loop {
        terminal.draw(|frame| render(frame, cpu, &mut state, bus))?;

        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') => should_quit = true,
                        KeyCode::Char('n') => {
                            cpu.clock_tick(bus);
                            state.manual_selection = None;
                            state.stack_manual_scroll = None;
                        },
                        KeyCode::Char('r') => {
                            state.running = !state.running;
                            state.manual_selection = None;
                            state.stack_manual_scroll = None;
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
                Event::Mouse(mouse) => {
                    let hit = |area: Rect| {
                        mouse.column >= area.x && mouse.column < area.x + area.width
                            && mouse.row >= area.y && mouse.row < area.y + area.height
                    };

                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            if hit(state.memory_area) {
                                state.memory_scroll_row = (state.memory_scroll_row + 3).min(4096usize.saturating_sub(1));
                            } else if hit(state.stack_area) {
                                let cur = state.stack_manual_scroll.unwrap_or(state.stack_table_state.offset());
                                state.stack_manual_scroll = Some((cur + 3).min(255));
                            } else if hit(state.opcode_area) {
                                let current = state.manual_selection.unwrap_or_else(|| {
                                    state.opcode_table_state.selected().unwrap_or(0)
                                });
                                let max = state.total_rows.saturating_sub(1);
                                state.manual_selection = Some((current + 3).min(max));
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if hit(state.memory_area) {
                                state.memory_scroll_row = state.memory_scroll_row.saturating_sub(3);
                            } else if hit(state.stack_area) {
                                let cur = state.stack_manual_scroll.unwrap_or(state.stack_table_state.offset());
                                state.stack_manual_scroll = Some(cur.saturating_sub(3));
                            } else if hit(state.opcode_area) {
                                let current = state.manual_selection.unwrap_or_else(|| {
                                    state.opcode_table_state.selected().unwrap_or(0)
                                });
                                state.manual_selection = Some(current.saturating_sub(3));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if should_quit {
            break;
        }

        if state.running && !cpu.halted {
            let delay_ms = 1000 / state.instructions_per_second.max(1);
            thread::sleep(Duration::from_millis(delay_ms as u64));
            cpu.clock_tick(bus);
            state.manual_selection = None;
            state.stack_manual_scroll = None;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    let _ = terminal.show_cursor();

    Ok(())
}