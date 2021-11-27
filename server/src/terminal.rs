use common::*;
use crossbeam_channel::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use tui_logger::{TuiLoggerTargetWidget, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};
use std::{
    io::{self, stdin, Stdout},
    thread::spawn,
};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, FromPrimitive)]
#[repr(usize)]
enum TerminalTab {
    #[default]
    Performance,
    Log,
    Info,
}
impl TerminalTab {
    const LEN: usize = TerminalTab::Info as usize + 1;
    const TITLES: [&'static str; TerminalTab::LEN] = ["Performance", "Log", "Info"];
}

pub struct Terminal {
    backend_terminal: tui::Terminal<TermionBackend<RawTerminal<Stdout>>>,
    input_receiver: Receiver<Key>,
    current_tab: TerminalTab,
    log_state: TuiWidgetState,
}
impl Terminal {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut backend_terminal = tui::Terminal::new(backend)?;
        backend_terminal.clear()?;

        let (input_sender, input_receiver) = unbounded();

        spawn(move || input_loop(input_sender));

        Ok(Self {
            backend_terminal,
            input_receiver,
            current_tab: TerminalTab::Performance,
            log_state: TuiWidgetState::default(),
        })
    }

    pub fn update(&mut self, stop_main: &mut bool) {
        // Handle inputs.
        while let Ok(key) = self.input_receiver.try_recv() {
            if let Key::Char(c) = key {
                if c == '\t' {
                    self.current_tab = TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
                    continue;
                }
            }
            if Key::Esc == key {
                // TODO: Ask to confirm.
                *stop_main = true;
                break;
            }

            match self.current_tab {
                TerminalTab::Performance => {}
                TerminalTab::Log => {
                    match key {
                        Key::Backspace => todo!(),
                        Key::Left => self.log_state.transition(&TuiWidgetEvent::LeftKey),
                        Key::Right => self.log_state.transition(&TuiWidgetEvent::RightKey),
                        Key::Up => self.log_state.transition(&TuiWidgetEvent::UpKey),
                        Key::Down => self.log_state.transition(&TuiWidgetEvent::DownKey),
                        Key::Home => todo!(),
                        Key::End => todo!(),
                        Key::PageUp => todo!(),
                        Key::PageDown => todo!(),
                        Key::BackTab => todo!(),
                        Key::Delete => todo!(),
                        Key::Insert => todo!(),
                        Key::F(_) => todo!(),
                        Key::Char(_) => todo!(),
                        Key::Alt(_) => todo!(),
                        Key::Ctrl(_) => todo!(),
                        Key::Null => todo!(),
                        Key::Esc => todo!(),
                        Key::__IsNotComplete => todo!(),
                    }
                }
                TerminalTab::Info => {}
            }
        }

        // Draw.
        let _ = self.backend_terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                // .margin(5)
                .constraints([Constraint::Length(3), Constraint::Percentage(60), Constraint::Min(8)].as_ref())
                .split(size);

            // Tabs titles.
            let tab_titles = TerminalTab::TITLES.into_iter().map(|s| Spans::from(s)).collect();
            let tabs = Tabs::new(tab_titles)
                .select(self.current_tab.into())
                .highlight_style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(tabs, chunks[0]);

            // Current tab.
            match self.current_tab {
                TerminalTab::Performance => {
                    let block = Block::default().borders(Borders::ALL);

                    frame.render_widget(block, chunks[1]);
                }
                TerminalTab::Log => {
                    let log = TuiLoggerTargetWidget::default()
                    .state(&self.log_state)
                    .block(Block::default().borders(Borders::ALL));

                    frame.render_widget(log, chunks[1]);
                }
                TerminalTab::Info => {
                    let text = vec![
                        Spans::from(format!("version: {}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)),
                        Spans::from(Span::styled("This is a line   ", Style::default().fg(Color::Red))),
                        Spans::from(Span::styled("This is a line", Style::default().bg(Color::Blue))),
                        Spans::from(Span::styled(
                            "This is a longer line",
                            Style::default().add_modifier(Modifier::CROSSED_OUT),
                        )),
                        Spans::from(Span::styled(
                            "This is a line",
                            Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC),
                        )),
                    ];

                    let paragraph = Paragraph::new(text)
                        .alignment(Alignment::Left)
                        .block(Block::default().borders(Borders::ALL));

                    frame.render_widget(paragraph, chunks[1]);
                }

            };

            // Logs.
            let mut log = TuiLoggerWidget::default()
                .style_error(Style::default().fg(Color::Red))
                .style_warn(Style::default().fg(Color::Yellow))
                // .style_info(Style::default().fg(Color::Cyan))
                .style_debug(Style::default().fg(Color::Green))
                .style_trace(Style::default().fg(Color::Magenta))
                .block(Block::default().borders(Borders::ALL));
            log.state(&self.log_state);

            frame.render_widget(log, chunks[2]);
        });
    }

    pub fn clear(&mut self) {
        let _ = self.backend_terminal.clear();
    }
}

fn input_loop(input_sender: Sender<Key>) {
    for key_result in stdin().keys() {
        match key_result {
            Ok(key) => {
                if input_sender.send(key).is_err() {
                    break;
                }
            }
            Err(err) => {
                debug!("Error while reading keys from stdin: {:?}", err);
            }
        }
    }
}
