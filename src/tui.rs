use ratatui::{
    Frame, Terminal, backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};

use ratatui::widgets::{Table, Row, Block, TableState};
use ratatui::style::{Style, Modifier};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use std::io;

use crate::cpu::CPU;
use crate::bus::Bus;

use crate::disasm::disassemble_range;

use std::{thread, time::Duration};

struct TuiState {
    opcode_list_scroll: usize,
    memory_scroll: usize,
    stack_scroll: usize,
    running: bool
}

const TARGET_HZ: u64 = 1_000_000; // 1 MHz
const NS_PER_CYCLE: u64 = 1_000_000_000 / TARGET_HZ; // nanosecond per cycle

pub fn render(frame: &mut Frame, cpu: &mut CPU, bus: &mut Bus, state: &TuiState) {
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

    let current_row = Row::new(vec![
        format!("{:04X}", cpu.pc),
        cpu.last_disasm.clone(),
    ]);


    let opcode_table = Table::new(
        vec![current_row],
        [Constraint::Length(7), Constraint::Min(10)],
    )
        .column_spacing(1)
        .header(header)
        .block(Block::bordered().title("Opcodes"));

    frame.render_widget(Block::bordered().title("Register"), left_chunk[0]);
    frame.render_widget(Block::bordered().title("TODO"), left_chunk[1]);
    frame.render_widget(opcode_table, main_chunk[0]);
    frame.render_widget(Block::bordered().title("Flags"), right_chunk[0]);
    frame.render_widget(Block::bordered().title("Memory"), right_chunk[1]);
    frame.render_widget(Block::bordered().title("Stack"), right_chunk[2]);
}

pub fn run(cpu: &mut CPU, bus: &mut Bus) -> io::Result<()> {
    let _ = enable_raw_mode();

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState {
        opcode_list_scroll: 0,
        memory_scroll: 0,
        stack_scroll: 0,
        running: false
    };

    loop {
        terminal.draw(|frame| render(frame, &mut *cpu, &mut *bus, &state))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('n') => cpu.clock_tick(bus),
                    KeyCode::Char('r') => state.running = !state.running,
                    _ => {}
                }
            }
        }

        if state.running && !cpu.halted {
            let delay_ns = NS_PER_CYCLE * cpu.last_cycles as u64;
            thread::sleep(Duration::from_nanos(delay_ns));

            cpu.clock_tick(bus);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = terminal.show_cursor();

    Ok(())
}