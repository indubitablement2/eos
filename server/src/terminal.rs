use common::*;
use crossbeam_channel::*;
use num_enum::{FromPrimitive, IntoPrimitive};
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
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Tabs},
};
use tui_logger::{TuiLoggerTargetWidget, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

use crate::Metascape;

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
    const TITLES: [&'static str; TerminalTab::LEN + 1] = ["Performance", "Log", "Info", "?"];
}

pub struct Terminal {
    backend_terminal: tui::Terminal<TermionBackend<RawTerminal<Stdout>>>,
    input_receiver: Receiver<Key>,
    current_tab: TerminalTab,
    help: bool,
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
            help: false,
            log_state: TuiWidgetState::default(),
        })
    }

    pub fn update(&mut self, stop_main: &mut bool, metascape: &mut Metascape) {
        // Handle inputs.
        while let Ok(key) = self.input_receiver.try_recv() {
            // Check if we are in help mode.
            if self.help == true {
                if let Key::Char(c) = key {
                    if c == '\t' {
                        self.help = false;
                        self.current_tab = TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
                    } else if c == '?' {
                        self.help = false;
                    }
                } else if Key::Esc == key {
                    self.help = false;
                }
                continue;
            }

            // Handle keys based on selected tab.
            match self.current_tab {
                TerminalTab::Performance => {}
                TerminalTab::Log => match key {
                    Key::Left => self.log_state.transition(&TuiWidgetEvent::LeftKey),
                    Key::Right => self.log_state.transition(&TuiWidgetEvent::RightKey),
                    Key::Up => self.log_state.transition(&TuiWidgetEvent::UpKey),
                    Key::Down => self.log_state.transition(&TuiWidgetEvent::DownKey),
                    Key::PageUp => self.log_state.transition(&TuiWidgetEvent::NextPageKey),
                    Key::PageDown => self.log_state.transition(&TuiWidgetEvent::PrevPageKey),
                    Key::Char(c) => match c {
                        'h' => self.log_state.transition(&TuiWidgetEvent::HideKey),
                        'f' => self.log_state.transition(&TuiWidgetEvent::FocusKey),
                        '-' => self.log_state.transition(&TuiWidgetEvent::MinusKey),
                        '+' => self.log_state.transition(&TuiWidgetEvent::PlusKey),
                        ' ' => self.log_state.transition(&TuiWidgetEvent::SpaceKey),
                        _ => (),
                    },
                    Key::Esc => {
                        self.log_state.transition(&TuiWidgetEvent::EscapeKey);
                    }
                    _ => (),
                },
                TerminalTab::Info => {}
            }

            // Special keys need to be paired with continue or they will be handled here too.
            if let Key::Char(c) = key {
                if c == '\t' {
                    self.current_tab = TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
                } else if c == '?' {
                    self.help = true;
                }
            } else if Key::Esc == key {
                // TODO: Ask to confirm.
                *stop_main = true;
                break;
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
            let mut tabs = Tabs::new(tab_titles)
                .highlight_style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            // Select ? if we are in help mode.
            if self.help {
                tabs = tabs.select(TerminalTab::LEN);
            } else {
                tabs = tabs.select(self.current_tab.into());
            }
            frame.render_widget(tabs, chunks[0]);

            // Current tab.
            if self.help {
                match self.current_tab {
                    TerminalTab::Performance => {}
                    TerminalTab::Log => {
                        let text = vec![
                            Spans::from("| h        | Toggles target selector widget hidden/visible."),
                            Spans::from("| f        | Toggle focus on the selected target only."),
                            Spans::from("| UP       | Select previous target in target selector widget."),
                            Spans::from("| DOWN     | Select next target in target selector widget."),
                            Spans::from("| LEFT     | Reduce SHOWN (!) log messages by one level."),
                            Spans::from("| RIGHT    | Increase SHOWN (!) log messages by one level."),
                            Spans::from("| -        | Reduce CAPTURED (!) log messages by one level."),
                            Spans::from("| +        | Increase CAPTURED (!) log messages by one level."),
                            Spans::from("| PAGEUP   | Enter Page Mode and scroll approx. half page up in log history."),
                            Spans::from("| PAGEDOWN | Only in page mode: scroll 10 events down in log history."),
                            Spans::from("| ESCAPE   | Exit page mode and go back to scrolling mode."),
                            Spans::from("| SPACE    | Toggles hiding of targets, which have logfilter set to off."),
                        ];
                        let paragraph = Paragraph::new(text)
                            .alignment(Alignment::Left)
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(paragraph, chunks[1]);
                    }
                    TerminalTab::Info => todo!(),
                }
            } else {
                match self.current_tab {
                    TerminalTab::Performance => {
                        let block = Block::default().borders(Borders::ALL);
                        frame.render_widget(block, chunks[1]);
                    }
                    TerminalTab::Log => {
                        let log = TuiLoggerTargetWidget::default()
                            .state(&self.log_state)
                            .highlight_style(Style::default().fg(Color::Yellow))
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(log, chunks[1]);
                    }
                    TerminalTab::Info => {
                        let text = vec![
                            Spans::from(format!(
                                "version: {}.{}.{}",
                                VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH
                            )),
                            Spans::from(format!("{:?}", metascape.get_addresses())),
                        ];
                        let paragraph = Paragraph::new(text)
                            .alignment(Alignment::Left)
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(paragraph, chunks[1]);
                    }
                };
            }

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
