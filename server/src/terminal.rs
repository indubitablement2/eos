use crossbeam_channel::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{
    io::{self, stdin, Stdout},
    str::FromStr,
    thread::spawn,
};
use termion::{
    event::{self, Key},
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

pub fn help() {
    println!("
    help: Show this text.
    exit: Save and shutdown server.
    addressses, addr, address: Show server udp/tcp address.
    log: Change log level. Log less than info are disabled for release build. Example: 'log trace ./main' will set main to trace. Default: info
    ");
}

#[derive(Debug, Clone)]
pub enum Commands {
    Test(bool),
    Help,
    Exit,
    Addressses,
    Log(String),
}
impl FromStr for Commands {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_lowercase();
        let mut iter = lower_case.split_whitespace();

        match iter.next().unwrap_or_default() {
            "test" => Ok(Commands::Test(
                iter.next().unwrap_or_default().parse().unwrap_or_default(),
            )),
            "exit" => Ok(Commands::Exit),
            "help" => Ok(Commands::Help),
            "addressses" => Ok(Commands::Addressses),
            "addr" => Ok(Commands::Addressses),
            "address" => Ok(Commands::Addressses),
            "log" => Ok(Commands::Log(iter.collect())),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, FromPrimitive)]
#[repr(usize)]
enum TerminalTab {
    #[default]
    Performance,
    Info,
}
impl TerminalTab {
    const LEN: usize = TerminalTab::Info as usize + 1;
    const TITLES: [&'static str; TerminalTab::LEN] = ["Performance", "Info"];
}

pub struct Terminal {
    backend_terminal: tui::Terminal<TermionBackend<RawTerminal<Stdout>>>,
    input_receiver: Receiver<Key>,
    current_tab: TerminalTab,
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
        })
    }

    pub fn render(&mut self) {
        // Handle inputs.
        self.input_receiver.try_iter().for_each(|key| {
            match key {
                Key::Left => todo!(),
                Key::Right => todo!(),
                Key::Up => todo!(),
                Key::Down => todo!(),
                Key::Char(c) => match c {
                    '\t' => {
                        self.current_tab = TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
                    }
                    _ => {}
                },
                // Key::Backspace => todo!(),
                // Key::Home => todo!(),
                // Key::End => todo!(),
                // Key::PageUp => todo!(),
                // Key::PageDown => todo!(),
                // Key::BackTab => todo!(),
                // Key::Delete => todo!(),
                // Key::Insert => todo!(),
                // Key::F(_) => todo!(),
                // Key::Alt(_) => todo!(),
                // Key::Ctrl(_) => todo!(),
                // Key::Esc => todo!(),
                _ => {}
            }
        });

        // Draw.
        let _ = self.backend_terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                // .margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(10)].as_ref())
                .split(size);

            // Tabs titles.
            let tab_titles = TerminalTab::TITLES.into_iter().map(|s| Spans::from(s)).collect();
            let tabs = Tabs::new(tab_titles)
                .select(self.current_tab.into())
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(tabs, chunks[0]);

            // Current tab.
            match self.current_tab {
                TerminalTab::Performance => {
                    let block = Block::default().borders(Borders::ALL);
                    frame.render_widget(block, chunks[1]);
                }
                TerminalTab::Info => {
                    let text = vec![
                        Spans::from("This is a line "),
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
            let log = Block::default().borders(Borders::ALL);
            frame.render_widget(log, chunks[2]);
        });
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
