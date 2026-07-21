use ansi_to_tui::IntoText;

use ratatui::{
    backend::CrosstermBackend,
    Frame,
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Rect,
    },
    style::{
        Color,
        Modifier,
        Style,
        Stylize,
    },
    Terminal,
    text::{
        Line,
        Span,
    },
    widgets::{
        Block,
        Gauge,
        List,
        ListItem,
        ListState,
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
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event::{
            self,
        },
        KeyCode::{
            self,
        },
        MouseEventKind,
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
    },
    fs,
    io,
    path::{
        Path,
        PathBuf
    },
    thread,
    time::Duration,
};

use crate::{
    cpu::CPU,
    bus::Bus,
    disasm::{
        DisasmLine,
        disassemble_range,
    }
};

#[derive(PartialEq)]
enum AppScreen {
    Emulator,
    Home,
}

#[derive(PartialEq)]
enum HomeFocus {
    FileList,
    Speed,
    StartAddr,
    StartButton
}

struct TuiState {
    current_dir: PathBuf,
    screen: AppScreen,
    show_settings: bool,

    available_files: Vec<String>,
    file_list_state: ListState,
    home_focus: HomeFocus,
    start_addr_input: String,

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
}

const SETTING_LOGO_ANSI: &str = "
[0;90;1m▄[0;37m▄▄▄▄▄▄ [0;90;1m▄[0;37m▄▄ [0;90;1m▄[0;37m▄▄ [0;90;1m▄[0;37m▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄       [0;90;1m▄[0;37m▄▄▄▄  [0;90;1m▄[0;37m▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄▄▄ [0;90;1m▄[0;37m▄▄▄▄ [0m
[0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀  [0;97;1;47m░[0;37m██       [0;97;1;47m░[0;37m██▄▄▄  [0;97;1;47m▓[0;37m██ [0;97;1m▀[0;37m▀ [0;97;1;47m░[0;37m██ [0;97;1;47m░[0;37m██ [0;97;1m▀[0;37m▀ [0;97;1;47m▒[0;37m██[0m
[0;97;1;47m▓[0;37m██▀[0;97;1;47m▄[0;37m█▄ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀▀[0;97;1;47m▄[0;37m██  [0;97;1;47m▒[0;37m██       [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1m▀[0;37m▀▀[0;97;1;47m▄[0;37m█▄ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██ [0;97;1;47m▒[0;37m██▀▀ [0m
[0;97;1m▀[0;37m▀▀ [0;97;1m▀[0;37m▀▀ [0;97;1m▀[0;37m▀▀▀▀▀▀ [0;90;1m▀[0;37m▀▀▀▀▀  [0;97;1m▀[0;37m▀▀        [0;97;1m▀[0;37m▀▀▀▀  [0;97;1m▀[0;37m▀▀▀▀  [0;97;1m▀[0;37m▀▀▀▀▀▀ [0;97;1m▀[0;37m▀▀▀▀▀[0m
";

const HOME_LOGO_ANSI: &str = "
[0;37m [0;97;1m┌───────┐[0;37m [0;97;1m┌───┐[0;37m [0;97;1m┌─┐[0;37m [0;97;1m┌───────┐[0;37m [0;97;1m┌───────┐[0;37m     [0;97;1m┌───────┐[0;37m [0;97;1m┌───────┐[0;37m [0;97;1m┌───────┐[0;37m [0;97;1m┌───────┐[0m
[0;34m═[0;97;1m│[0;90;1m∙[0;37m  ╒═[0;97;1m╕[0;90;1m∙[0;37m│[0;34m═[0;97;1m│[0;90;1m∙[0;37m  │[0;34m═[0;97;1m│[0;90;1m∙[0;37m│[0;34m═[0;37m│[0;90;1m∙[0;97;1m╒[0;37m═════[0;97;1m╛[0;34m═[0;37m╘═[0;97;1m╕[0;90;1m∙[0;37m [0;34m·[0;37m╒═╛    [0;34m═[0;97;1m│[0;90;1m∙[0;37m  ╒═══╛[0;34m═[0;37m│[0;90;1m∙[0;97;1m╒[0;37m═════[0;97;1m╛[0;34m═[0;97;1m│[0;90;1m∙[0;37m  ╒═[0;97;1m╕[0;90;1m∙[0;37m│[0;34m═[0;97;1m╘[0;37m═════[0;97;1m╕[0;90;1m∙[0;37m│[0m
[0;37;41m [0;97;1m│[0;37m   └[0;97;1m─┘┌┘[0;91;1;41m░[0;97;1m│[0;37m   │[0;91;1;41m█[0;97;1m│[0;37m │[0;91;1;41m▓[0;37m│ [0;97;1m└─────┐[0;91;1;41m░░▒[0;97;1m│[0;37m   │[0;91;1m▒[0;91;1;41m░[0;37m    [0;37;41m [0;97;1m│[0;37m   └[0;97;1m───┐[0;91;1;41m▓[0;37m│ [0;97;1m└─────┐[0;91;1;41m░[0;97;1m│[0;37m   │[0;91;1;41m█[0;97;1m│[0;37m │[0;91;1;41m▓[0;97;1m┌─────┘[0;37m │[0m
[0;91;1;41m░[0;97;1m│[0;37m   ╒═╕[0;97;1m└┐[0;91;1;41m▒[0;97;1m│[0;37m   │[0;91;1;41m▓[0;97;1m│[0;37m │[0;91;1;41m█[0;37m╘═══╕  [0;90;1m∙[0;97;1m│[0;91;1;41m░░▒[0;97;1m│[0;37m   │[0;91;1;41m▒░[0;37m    [0;91;1;41m░[0;97;1m│[0;37m   [0;97;1m┌─┐[0;34m·[0;37m│[0;91;1;41m█[0;37m╘═══╕  [0;90;1m∙[0;97;1m│[0;91;1;41m▒[0;97;1m│[0;37m   │[0;91;1;41m▓[0;97;1m│[0;37m │[0;91;1;41m█[0;97;1m│[0;90;1m∙[0;37m  ╒═══╛[0m
[0;91;1;41m▒[0;97;1m│[0;37m   │[0;91;1;41m░[0;97;1m│[0;37m │[0;91;1;41m▓[0;97;1m│[0;37m   └[0;97;1m─┘[0;37m │[0;91;1;41m▓[0;97;1m┌───┘[0;37m   [0;97;1m│[0;91;1;41m░░▒[0;97;1m│[0;37m   │[0;91;1;41m▒░[0;37m    [0;91;1;41m▒[0;97;1m│[0;37m   │[0;91;1;41m░[0;97;1m│[0;37m │[0;91;1;41m▓[0;97;1m┌───┘[0;37m   [0;97;1m│[0;91;1;41m▓[0;97;1m│[0;37m   └[0;97;1m─┘[0;37m │[0;91;1;41m▓[0;97;1m│[0;37m   [0;97;1m└───┐[0m
[0;34m═[0;97;1m│[0;90;1m∙[0;37m  │[0;34m═[0;97;1m│[0;90;1m∙[0;37m│[0;34m═[0;97;1m│[0;90;1m∙[0;37m     [0;90;1m∙[0;37m│[0;34m═[0;37m│[0;90;1m∙[0;37m     [0;90;1m∙[0;97;1m│[0;34m═══[0;97;1m│[0;90;1m∙[0;37m [0;34m·[0;37m│[0;34m══[0;37m    [0;34m═[0;97;1m│[0;90;1m∙[0;37m  ╘═[0;97;1m╛[0;90;1m∙[0;37m│[0;34m═[0;37m│[0;90;1m∙[0;37m     [0;90;1m∙[0;97;1m│[0;34m═[0;97;1m│[0;90;1m∙[0;37m     [0;90;1m∙[0;37m│[0;34m═[0;97;1m│[0;90;1m∙[0;37m     [0;90;1m∙[0;37m│[0m
[0;37m [0;97;1m╘[0;37m═══╛ [0;97;1m╘[0;37m═╛ [0;97;1m╘[0;37m═══════╛ ╘═══════[0;97;1m╛[0;37m   [0;97;1m╘[0;37m═══╛       [0;97;1m╘[0;37m═══════╛ ╘═══════[0;97;1m╛[0;37m [0;97;1m╘[0;37m═══════╛ [0;97;1m╘[0;37m═══════╛[0m
";

const SCREEN_ADDR: u16 = 0x0200;
const SCREEN_WIDTH: usize = 32;
const SCREEN_HEIGHT: usize = 32;

const IPS: u32 = 7000; // instruction per second

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

fn flag_span(label: &str, set: bool) -> Span<'static> {
    let style = if set {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    Span::styled(format!("{} ", label), style)
}

fn load_directory_contents(path: &Path) -> Vec<String> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    dirs.push("..".to_string()); // always add ../ path

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    dirs.push(format!("{}/", name));
                } else {
                    files.push(name);
                }
            }
        }
    }

    dirs.sort_by(|a, b| {
        if a == ".." {
            std::cmp::Ordering::Less
        } else if b == ".." {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });


    files.sort();

    dirs.extend(files);
    dirs
}

fn palette_color(index: u8) -> Color {
    match index & 0x0F {
        0 => Color::Rgb(0, 0, 0),
        1 => Color::Rgb(255, 255, 255),
        2 => Color::Rgb(255, 0, 0),
        3 => Color::Rgb(0, 255, 255),
        4 => Color::Rgb(255, 0, 255),
        5 => Color::Rgb(0, 255, 0),
        6 => Color::Rgb(0, 0, 255),
        7 => Color::Rgb(255, 255, 0),
        8 => Color::Rgb(255, 128, 0),
        9 => Color::Rgb(128, 64, 0),
        10 => Color::Rgb(255, 64, 64),
        11 => Color::Rgb(32, 32, 32),
        12 => Color::Rgb(128, 128, 128),
        13 => Color::Rgb(64, 64, 255),
        14 => Color::Rgb(64, 255, 64),
        _ => Color::Rgb(200, 200, 200),
    }
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
        .block(Block::bordered().title(" Flags "));

    frame.render_widget(flag_widget, area);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &TuiState) {
    let spans = if state.show_settings {
        vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Adjust Speed  "),
            Span::styled(" ESC/? ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Close "),
        ]
    } else {
        vec![
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Step  "),
            Span::styled(" Space ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Run/Pause  "),
            Span::styled(" R ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Reset  "),
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Scroll  "),
            Span::styled(" ? ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Settings  "),
            Span::styled(" Q ", Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit "),
        ]
    };

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

fn render_home(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // logo
            Constraint::Min(0), // content
        ])
        .split(area);

    let mut text = HOME_LOGO_ANSI.into_text().unwrap_or_default();
    for line in text.lines.iter_mut() {
        for span in line.spans.iter_mut() {
            if span.style.bg == Some(Color::Black) {
                span.style.bg = Some(Color::Reset);
            }
        }
    }

    let logo_widget = Paragraph::new(text).centered();
    frame.render_widget(logo_widget, main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .margin(2)
        .split(main_layout[1]);

    let file_style = if state.home_focus == HomeFocus::FileList {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = state.available_files.iter().map(|item| {
        if item == ".." {
            ListItem::new("  ../ (Parent Directory) ").style(Style::default().fg(Color::Cyan))
        } else if item.ends_with('/') {
            ListItem::new(format!("  {} ", item)).style(Style::default().fg(Color::Blue))
        } else {
            ListItem::new(format!("  {} ", item))
        }
    }).collect();

    let file_list = List::new(items)
        .block(Block::bordered().title(" Select File ").border_style(file_style))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(file_list, content_layout[0], &mut state.file_list_state);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // file path display
            Constraint::Length(3), // start addr
            Constraint::Length(3), // speed slider
            Constraint::Length(2), // spacer
            Constraint::Length(5), // start btn
        ])
        .split(content_layout[1]);

    let selected_file_display = state
            .file_list_state
            .selected()
            .and_then(|i| state.available_files.get(i))
            .and_then(|name| {
                if name == ".." || name.ends_with("/") {
                    None
                } else {
                    Some(state.current_dir.join(name).display().to_string())
                }
            })
            .unwrap_or_else(|| "No file selected".to_string());

    let path_display = Paragraph::new(format!(" {}", selected_file_display))
        .block(Block::bordered().title(" Selected File "));

    frame.render_widget(path_display, right_layout[0]);

    let addr_style = if state.home_focus == HomeFocus::StartAddr {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let addr_display = Paragraph::new(format!(" 0x{}", state.start_addr_input))
        .block(Block::bordered().title(" Start Address (Hex) ").border_style(addr_style));

    frame.render_widget(addr_display, right_layout[1]);

    let speed_style = if state.home_focus == HomeFocus::Speed {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let ratio = state.instructions_per_second as f64 / 10_000.0;
    let speed_gauge = Gauge::default()
        .block(Block::bordered().title(" Emulation Speed ").border_style(speed_style))
        .gauge_style(Style::default().fg(if state.home_focus == HomeFocus::Speed { Color::DarkGray } else { Color::Black }))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(format!("{} IPS", state.instructions_per_second));

    frame.render_widget(speed_gauge, right_layout[2]);

    let btn_area = centered_rect(50, 100, right_layout[4]);
    let (btn_style, text_style, shadow_offset) = if state.home_focus == HomeFocus::StartButton {
        (Style::default().fg(Color::Yellow), Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD), 0)
    } else {
        (Style::default().fg(Color::White), Style::default().fg(Color::White).add_modifier(Modifier::BOLD), 1)
    };

    if shadow_offset > 0 {
        let shadow_area = Rect {
            x: btn_area.x - 1,
            y: btn_area.y - 1,
            ..btn_area
        };
        frame.render_widget(Block::default().fg(Color::DarkGray), shadow_area);
    }

    let face_area = Rect {
        x: btn_area.x,
        y: btn_area.y,
        ..btn_area
    };
    frame.render_widget(ratatui::widgets::Clear, face_area);

    let start_btn = Paragraph::new("Start Emulator")
        .alignment(Alignment::Center)
        .style(text_style)
        .block(Block::bordered().border_type(ratatui::widgets::BorderType::Thick).border_style(btn_style));

    let text_area = Rect {
        y: face_area.y + 1,
        height: face_area.height - 2,
        ..face_area
    };

    frame.render_widget(start_btn, text_area);

    let help_text = Paragraph::new(" [TAB] Change Focus   [↑↓] Select/Adjust   [ENTER] Confirm ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    let mut footer_area = area;
    footer_area.y = area.height.saturating_sub(2);
    footer_area.height = 1;
    frame.render_widget(help_text, footer_area);
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
        .block(Block::bordered().title(" Memory "));

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

    let title_string = format!(" Opcodes - \"{}\" ", display_name);

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

fn render_screen(frame: &mut Frame, area: Rect, bus: &Bus) {
    let block = Block::bordered().title(" Screen ");
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
        Line::from(Span::styled(" ↑/↓ to adjust", Style::default().fg(Color::DarkGray))).centered(),
        Line::from(""),
        Line::from(vec![
            Span::raw("Github: "),
            Span::styled("github.com/wirenux/rust-6502", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
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
        .block(Block::bordered().title(" Stack "));

    frame.render_stateful_widget(stack_table, area, &mut state.stack_table_state);

    let visible_height = area.height.saturating_sub(3) as usize;
    let max_offset = 256usize.saturating_sub(visible_height);

    let mut scrollbar_state = ScrollbarState::new(max_offset).position(state.stack_table_state.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn render_emulator(frame: &mut Frame, cpu: &mut CPU, state: &mut TuiState, bus: &mut Bus) {
    let screen_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let outer_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // left: register/todo
            Constraint::Percentage(80), // rest: opcodes + flags/memory/stack
        ])
        .split(screen_chunk[0]);

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
    render_footer(frame, screen_chunk[1], state);

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

pub fn run(cpu: &mut CPU, bus: &mut Bus, disasm_start: u16, file_path: Option<String>) -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let disasm_lines = disassemble_range(bus, disasm_start, 2000); // make cache

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let files = load_directory_contents(&current_dir);

    let initial_screen = if file_path.is_some() {
        AppScreen::Emulator
    } else {
        AppScreen::Home
    };

    let mut state = TuiState {
        current_dir,
        screen: initial_screen,
        show_settings: false,

        home_focus: HomeFocus::FileList,
        available_files: files,
        file_list_state: ListState::default().with_selected(Some(0)),
        start_addr_input: format!("{:04X}", disasm_start),

        disasm_lines,
        filename: file_path.unwrap_or_default(),
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
    };

    let mut last_frame_time = std::time::Instant::now();
    let mut should_quit = false;

    loop {
        terminal.draw(|frame| {
            match state.screen {
                AppScreen::Home => render_home(frame, &mut state),
                AppScreen::Emulator => render_emulator(frame, cpu, &mut state, bus),
            }
        })?;

        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('q') && state.home_focus != HomeFocus::StartAddr {
                        should_quit = true;
                    }

                    match state.screen {
                        AppScreen::Home => {
                            match key.code {
                                KeyCode::Tab => {
                                    state.home_focus = match state.home_focus {
                                        HomeFocus::FileList => HomeFocus::StartAddr,
                                        HomeFocus::StartAddr => HomeFocus::Speed,
                                        HomeFocus::Speed => HomeFocus::StartButton,
                                        HomeFocus::StartButton => HomeFocus::FileList,
                                    }
                                },
                                KeyCode::Up => {
                                    if state.home_focus == HomeFocus::FileList && !state.available_files.is_empty() {
                                        let i = state.file_list_state.selected().unwrap_or(0);
                                        let new_i = i.saturating_sub(1);
                                        state.file_list_state.select(Some(new_i));

                                        let file_name = &state.available_files[new_i].to_lowercase();
                                        if file_name.contains("demo") && file_name.ends_with(".bin") {
                                            state.start_addr_input = "C000".to_string();
                                        }
                                    }
                                },
                                KeyCode::Down => {
                                    if state.home_focus == HomeFocus::FileList && !state.available_files.is_empty() {
                                        let i = state.file_list_state.selected().unwrap_or(0);
                                        let new_i = (i + 1).min(state.available_files.len().saturating_sub(1));
                                        state.file_list_state.select(Some(new_i));

                                        let file_name = &state.available_files[new_i].to_lowercase();
                                        if file_name.contains("demo") && file_name.ends_with(".bin") {
                                            state.start_addr_input = "C000".to_string();
                                        }
                                    }
                                },
                                KeyCode::Left => {
                                    if state.home_focus == HomeFocus::Speed {
                                        state.instructions_per_second = state.instructions_per_second.saturating_sub(100);
                                    }
                                },
                                KeyCode::Right => {
                                    if state.home_focus == HomeFocus::Speed {
                                        state.instructions_per_second = state.instructions_per_second.saturating_add(100).min(10_000);
                                    }
                                },
                                KeyCode::Backspace => {
                                    if state.home_focus == HomeFocus::StartAddr {
                                        state.start_addr_input.pop();
                                    }
                                },
                                KeyCode::Char(c) => {
                                    if state.home_focus == HomeFocus::StartAddr {
                                        if c.is_ascii_hexdigit() && state.start_addr_input.len() < 4 {
                                            state.start_addr_input.push(c.to_ascii_uppercase());
                                        }
                                    }
                                },
                                KeyCode::Enter => {
                                    match state.home_focus {
                                        HomeFocus::StartButton => {
                                            if let Ok(addr) = u16::from_str_radix(&state.start_addr_input, 16) {
                                                if let Some(idx) = state.file_list_state.selected() {
                                                    if let Some(selected_name) = state.available_files.get(idx) {
                                                        let full_path = state.current_dir.join(selected_name);

                                                        if full_path.is_file() {
                                                            match fs::read(&full_path) {
                                                                Ok(program_bytes) => {
                                                                    state.filename = full_path.to_string_lossy().to_string();

                                                                    bus.load_rom(&program_bytes, addr);
                                                                    state.disasm_lines = disassemble_range(bus, addr, 2000);
                                                                    cpu.pc = addr;
                                                                    state.screen = AppScreen::Emulator;
                                                                }
                                                                Err(err) => {
                                                                    eprintln!("Failed to read file: {err}");
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        HomeFocus::FileList => {
                                            if let Some(selected_idx) = state.file_list_state.selected() {
                                                if let Some(selected_name) = state.available_files.get(selected_idx).cloned() {
                                                    if selected_name == ".." {
                                                        if let Some(parent) = state.current_dir.parent() {
                                                            state.current_dir = parent.to_path_buf();
                                                            state.available_files = load_directory_contents(&state.current_dir);
                                                            state.file_list_state.select(Some(0));
                                                        }
                                                    } else if selected_name.ends_with('/') {
                                                        let folder_name = selected_name.trim_end_matches('/');
                                                        state.current_dir = state.current_dir.join(folder_name);
                                                        state.available_files = load_directory_contents(&state.current_dir);
                                                        state.file_list_state.select(Some(0));
                                                    } else {
                                                        let full_file_path = state.current_dir.join(&selected_name);
                                                        state.filename = full_file_path.to_string_lossy().to_string();
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        },
                        AppScreen::Emulator => {
                            if state.show_settings {
                                match key.code {
                                    KeyCode::Char('?') | KeyCode::Esc => state.show_settings = false,
                                    KeyCode::Up => state.instructions_per_second = state.instructions_per_second.saturating_add(100),
                                    KeyCode::Down => state.instructions_per_second = state.instructions_per_second.saturating_sub(100),
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char('r') => {
                                        cpu.reset_cpu(bus);
                                        cpu.reset_stack(bus);
                                        cpu.reset_screen(bus);
                                        cpu.halted = false;
                                        state.manual_selection = None;
                                        state.running = false;
                                        state.stack_manual_scroll = None;
                                    },
                                    KeyCode::Enter => {
                                        if !cpu.halted {
                                            let prev_pc = cpu.pc;
                                            cpu.clock_tick(bus);
                                            if bus.read_ram(prev_pc) == 0x00 {
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
                                    KeyCode::Char(' ') => {
                                        state.running = !state.running;
                                        state.manual_selection = None;
                                        state.stack_manual_scroll = None;
                                    },
                                    KeyCode::Char('?') => state.show_settings = true,
                                    KeyCode::Up => {
                                        let current = state.manual_selection.unwrap_or_else(|| state.opcode_table_state.selected().unwrap_or(0));
                                        state.manual_selection = Some(current.saturating_sub(1));
                                    }
                                    KeyCode::Down => {
                                        let current = state.manual_selection.unwrap_or_else(|| state.opcode_table_state.selected().unwrap_or(0));
                                        state.manual_selection = Some((current + 1).min(state.total_rows.saturating_sub(1)));
                                    },
                                    KeyCode::Esc => {
                                        state.screen = AppScreen::Home;
                                        state.running = false;
                                    }
                                    _ => {}
                                }
                            }
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