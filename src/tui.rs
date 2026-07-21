use ansi_to_tui::IntoText;

use ratatui::{
    backend::{
        CrosstermBackend,
    },
    Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
        Rect,
    },
    style::{
        Color,
        Modifier,
        Style,
    },
    Terminal,
    text::{
        Line,
        Span,
    },
    widgets::{
        Block,
        Paragraph,
        Row,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        Table,
        TableState,
    },
};

use crossterm::{
    event::{
        DisableMouseCapture,
        EnableMouseCapture,
        Event::{
            self,
        },
        KeyCode::{
            self,
        },
        MouseEventKind,
        self,
    },
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use std::{
    collections::{
        HashMap,
        HashSet
    }, io, path::Path, thread, time::Duration,
};

use crate::{
    cpu::CPU,
    bus::Bus,
    disasm::{
        disassemble_range,
        DisasmLine
    }
};

struct TuiState {
    disasm_lines: Vec<DisasmLine>,
    filename: String,
    instructions_per_second: u32,
    manual_selection: Option<usize>,
    memory_area: Rect,
    memory_table_state: TableState,
    memory_scroll_row: usize,
    opcode_area: Rect,
    opcode_table_state: TableState,
    running: bool,
    stack_area: Rect,
    stack_table_state: TableState,
    stack_manual_scroll: Option<usize>,
    total_rows: usize,
    show_settings: bool,
}

const SETTING_LOGO_ANSI: &str = "
[0;90;1m▄[0;37m▄▄▄▄▄▄ [0;90;1m▄[0;37m▄▄ [0;90;1m▄[0;37m▄▄ [0;90;1m▄[0;37m▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄       [0;90;1m▄[0;37m▄▄▄▄  [0;90;1m▄[0;37m▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄ [0m
[0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀  [0;97;1;47m░[0;37m██       [0;97;1;47m░[0;37m██▄▄▄  [0;97;1;47m▓[0;37m██ [0;97;1m▀[0;37m▀ [0;97;1;47m░[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1m▀[0;37m▀ [0;97;1;47m▒[0;37m██[0m
[0;97;1;47m▓[0;37m██▀[0;97;1;47m▄[0;37m█▄ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀▀[0;97;1;47m▄[0;37m██  [0;97;1;47m▒[0;37m██       [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀▀[0;97;1;47m▄[0;37m█▄ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██▀▀ [0m
[0;97;1m▀[0;37m▀▀ [0;97;1m▀[0;37m▀▀ [0;97;1m▀[0;37m▀▀▀▀▀▀ [0;90;1m▀[0;37m▀▀▀▀▀  [0;97;1m▀[0;37m▀▀        [0;97;1m▀[0;37m▀▀▀▀  [0;97;1m▀[0;37m▀▀▀▀  [0;97;1m▀[0;37m▀▀▀▀▀▀ [0;97;1m▀[0;37m▀▀▀▀▀[0m
";

const SCREEN_ADDR: u16 = 0x0200;
const SCREEN_WIDTH: usize = 32;
const SCREEN_HEIGHT: usize = 32;

const IPS: u32 = 7000; // instruction per second

fn palette_color(index: u8) -> Color {
    match index & 0x0F {
        0 => Color::Black,
        1 => Color::White,
        2 => Color::Red,
        3 => Color::Cyan,
        4 => Color::Magenta,
        5 => Color::Green,
        6 => Color::Blue,
        7 => Color::Yellow,
        8 => Color::Rgb(255, 165, 0), // Orange
        9 => Color::Rgb(153, 76, 0), // Brown
        10 => Color::DarkGray,
        11 => Color::Gray,
        12 => Color::LightRed,
        13 => Color::LightBlue,
        14 => Color::LightGreen,
        _ => Color::Reset, // 15 transparent
    }
}

fn find_label_addr(lines: &[DisasmLine]) -> HashSet<u16> {
    let mut labels = HashSet::new();

    for line in lines {
        if let Some(addr_str) = line.text.split('$').nth(1) {
            let clean_str: String = addr_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if let Ok(val) = u16::from_str_radix(&clean_str, 16) {
                if line.text.starts_with("JSR") || line.text.starts_with("JMP") {
                    labels.insert(val);
                } else if line.text.starts_with('B') { // BEQ, BNE, BCC, BCS, BMI, BPL, BVC, BVS
                    if clean_str.len() <= 2 {
                        let offset = val as u8 as i8;
                        let target_addr = (line.addr as i32 + 2 + offset as i32) as u16;
                        labels.insert(target_addr);
                    }
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

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_logo(frame: &mut Frame, area: Rect) {
    let mut text = SETTING_LOGO_ANSI.into_text().unwrap_or_default();

    for line in text.lines.iter_mut() {
        for span in line.spans.iter_mut() {
            if span.style.bg == Some(Color::Black) {
                span.style.bg = Some(Color::Reset);
            }
        }
    }

    let widget = Paragraph::new(text).centered();
    frame.render_widget(widget, area);
}

fn render_settings_popup(frame: &mut Frame, area: Rect, state: &TuiState) {
    let popup_area = centered_rect(50, 40, area);

    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let block = Block::bordered().title(" Settings ").border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(inner);

    render_logo(frame, sections[0]);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Speed: "),
            Span::styled(format!("{} ips", state.instructions_per_second), Style::default().fg(Color::Green)),
        ]).centered(),
        Line::from(Span::styled("  ↑/↓ to adjust", Style::default().fg(Color::DarkGray))).centered(),
        Line::from(""),
        Line::from(vec![
            Span::raw("Github: "),
            Span::styled("github.com/wirenux/rust6502", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]).centered(),
        Line::from(vec![
            Span::raw("Stardance: "),
            Span::styled("stardance.hackclub.com/projects/34098", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]).centered(),
        Line::from(""),
        Line::from(Span::styled("Press ? or ESC to close", Style::default().fg(Color::DarkGray))).centered(),
    ];

    let text_widget = Paragraph::new(lines);
    frame.render_widget(text_widget, sections[1]);
}

fn render_popup_shadow(frame: &mut Frame, popup_area: Rect, full_area: Rect) {
    let shadow_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width,
        height: popup_area.height,
    }.intersection(full_area);

    let shadow = Block::default().style(Style::default().bg(Color::White).fg(Color::White));
    frame.render_widget(shadow, shadow_area);
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
        None => {
            if cpu.halted && state.opcode_table_state.selected().is_some() {
                state.opcode_table_state.selected()
            } else {
                addr_to_row.get(&cpu.pc).copied().or(state.opcode_table_state.selected())
            }
        }
    };

    state.opcode_table_state.select(selected_index);

    if state.manual_selection.is_none() {
        if let Some(idx) = selected_index {
            let desired_offset = idx.saturating_sub(5);
            *state.opcode_table_state.offset_mut() = desired_offset.min(max_offset);
        }
    }

    let display_name = Path::new(&state.filename)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| state.filename.clone());

    let title_string = format!("Opcodes - \"{}\"", display_name);

    let opcode_table = Table::new(rows, [
        Constraint::Length(9),
        Constraint::Min(10),
    ])
        .column_spacing(1)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .block(Block::bordered().title(title_string));

    frame.render_stateful_widget(opcode_table, area, &mut state.opcode_table_state);

    let mut scrollbar_state = ScrollbarState::new(max_offset)
        .position(state.opcode_table_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn render_screen(frame: &mut Frame, area: Rect, bus: &Bus) {
    let block = Block::bordered().title("Screen");
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let cell_cols = SCREEN_WIDTH;
    let cell_rows = SCREEN_HEIGHT / 2; // div by 2 because i use the unicode half block char

    let render_width = cell_cols.min(inner.width as usize);
    let render_height = cell_rows.min(inner.height as usize);

    let x_offset = (inner.width as usize).saturating_sub(cell_cols) / 2;
    let y_offset = (inner.height as usize).saturating_sub(cell_rows) / 2;

    let mut lines = Vec::with_capacity(render_height);

    for row in 0..render_height {
        let mut spans = Vec::with_capacity(render_width);
        let top_pixel_row = row * 2;
        let bottom_pixel_row = row * 2 + 1;

        for col in 0..render_width {
            let top_addr = SCREEN_ADDR + (top_pixel_row * SCREEN_WIDTH + col) as u16;
            let bottom_addr = SCREEN_ADDR + (bottom_pixel_row * SCREEN_HEIGHT + col) as u16;

            let top_color = palette_color(bus.read_ram(top_addr));
            let bottom_color = palette_color(bus.read_ram(bottom_addr));

            spans.push(Span::styled("▀", Style::default().fg(top_color).bg(bottom_color)));
        }

        lines.push(Line::from(spans));
    }

    let screen_widget = Paragraph::new(lines);

    let screen_area = Rect {
        x: inner.x + x_offset as u16,
        y: inner.y + y_offset as u16,
        width: render_width as u16,
        height: render_height as u16,
    };

    frame.render_widget(screen_widget, screen_area);
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
    render_stack(frame, left_chunk[2], cpu, bus, state);
    render_register(frame, left_chunk[1], cpu);
    render_opcodes(frame, main_chunk[0], cpu, state);
    render_memory(frame, right_chunk[0], bus, state);
    render_screen(frame, right_chunk[1], bus);

    state.memory_area = right_chunk[0];
    state.stack_area = left_chunk[2];
    state.opcode_area = main_chunk[0];

    if state.show_settings {
        let popup_area = centered_rect(50, 40, frame.area());

        render_popup_shadow(frame, popup_area, frame.area());
        frame.render_widget(ratatui::widgets::Clear, popup_area);
        render_settings_popup(frame, frame.area(), state);
    }
}

pub fn run(cpu: &mut CPU, bus: &mut Bus, disasm_start: u16, filename: &str) -> io::Result<()> {
    let _ = enable_raw_mode();

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let disasm_lines = disassemble_range(bus, disasm_start, 2000); // make cache

    let mut state = TuiState {
        disasm_lines,
        filename: filename.to_string(),
        instructions_per_second: IPS,
        manual_selection: None,
        memory_scroll_row: 0,
        memory_table_state: TableState::default(),
        memory_area: Rect::default(),
        opcode_area: Rect::default(),
        opcode_table_state: TableState::default(),
        running: false,
        stack_area: Rect::default(),
        stack_manual_scroll: None,
        stack_table_state: TableState::default(),
        total_rows: 0,
        show_settings: false,
    };

    let mut last_frame_time = std::time::Instant::now();
    let mut should_quit = false;

    loop {
        terminal.draw(|frame| render(frame, cpu, &mut state, bus))?;

        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if state.show_settings {
                        match key.code {
                            KeyCode::Char('q') => should_quit = true,
                            KeyCode::Char('?') | KeyCode::Esc => state.show_settings = false,
                            KeyCode::Up => {
                                state.instructions_per_second = state.instructions_per_second.saturating_add(100);
                            }
                            KeyCode::Down => {
                                state.instructions_per_second = state.instructions_per_second.saturating_sub(100);
                            },
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => should_quit = true,
                            KeyCode::Char('n') => {
                                if !cpu.halted {
                                    let prev_pc = cpu.pc;
                                    cpu.clock_tick(bus);

                                    if bus.read_ram(prev_pc) == 0x00 { // BRK
                                        let labels = find_label_addr(&state.disasm_lines);
                                        let (_, addr_to_row) = build_opcode_rows(&state.disasm_lines, &labels);
                                        if let Some(&row_idx) = addr_to_row.get(&prev_pc) {
                                            state.manual_selection = Some(row_idx);
                                        }
                                    } else {
                                        state.manual_selection = None;
                                    }
                                    state.stack_manual_scroll = None;
                                }
                            },
                            KeyCode::Char('r') => {
                                state.running = !state.running;
                                state.manual_selection = None;
                                state.stack_manual_scroll = None;
                            },
                            KeyCode::Char('?') => {
                                state.show_settings = true;
                            }
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
            let elapsed = last_frame_time.elapsed();
            last_frame_time = std::time::Instant::now();
            let instructions_to_run = (elapsed.as_secs_f64() * state.instructions_per_second as f64) as u32;

            for _ in 0..instructions_to_run {
                if cpu.halted { break; }

                let prev_pc = cpu.pc;
                cpu.clock_tick(bus);

                if bus.read_ram(prev_pc) == 0x00 { // BRK
                    state.running = false; // the emulator goes to HALTED mode

                    let labels = find_label_addr(&state.disasm_lines);
                    let (_, addr_to_row) = build_opcode_rows(&state.disasm_lines, &labels);
                    if let Some(&row_idx) = addr_to_row.get(&prev_pc) {
                        state.manual_selection = Some(row_idx);
                    }
                }
            }

            if state.running {
                state.manual_selection = None;
                state.stack_manual_scroll = None;
            }
        } else {
            last_frame_time = std::time::Instant::now();
        }

        thread::sleep(Duration::from_millis(16));
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    let _ = terminal.show_cursor();

    Ok(())
}