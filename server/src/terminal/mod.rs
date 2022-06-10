pub mod input;
pub mod performance;

use common::UPDATE_INTERVAL;
use crossbeam::channel::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use performance::PerformanceMetrics;
use std::{
    io::{self, Stdout},
    thread::spawn,
};
use termion::{
    event::Key,
    raw::{IntoRawMode, RawTerminal},
};
use tui::{
    backend::TermionBackend,
    layout::*,
    style::*,
    symbols::*,
    text::{Span, Spans},
    widgets::*,
};
use tui_logger::{TuiLoggerTargetWidget, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, FromPrimitive, Default,
)]
#[repr(usize)]
enum TerminalTab {
    #[default]
    Performance,
    Log,
    Info,
    /// Skipped over when pressing tab.
    Exit,
}
impl TerminalTab {
    const LEN: usize = TerminalTab::Info as usize + 1;
    const TITLES: [&'static str; TerminalTab::LEN + 2] =
        ["Performance", "Log", "Info", "Exit", "?"];
}

pub struct Terminal {
    backend_terminal: tui::Terminal<TermionBackend<RawTerminal<Stdout>>>,

    input_receiver: Receiver<Key>,
    /// How many update without input.
    /// If idle, automaticaly go to help mode.
    last_input: u32,

    log_state: TuiWidgetState,

    current_tab: TerminalTab,
    /// Display help for current tab.
    help: bool,

    need_redraw: bool,
    draw_interval: i32,
}
impl Terminal {
    /// Only draw once for every DRAW_INTERVAL update.
    const DRAW_INTERVAL: i32 = 2;
    /// Stop drawing if we did get any input for that many update.
    const IDLE_DELAY: u32 = 600;

    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut backend_terminal = tui::Terminal::new(backend)?;
        backend_terminal.clear()?;

        let (input_sender, input_receiver) = unbounded();

        spawn(move || input::input_loop(input_sender));

        Ok(Self {
            backend_terminal,
            input_receiver,
            current_tab: Default::default(),
            help: false,
            last_input: 0,
            need_redraw: true,
            log_state: Default::default(),
            draw_interval: Default::default(),
        })
    }

    /// Return if we should quit.
    pub fn update(&mut self, performance: &PerformanceMetrics) -> bool {
        self.last_input += 1;

        // Handle inputs.
        while let Ok(key) = self.input_receiver.try_recv() {
            if self.handle_inputs(key) {
                return true;
            }
            self.last_input = 0;
        }

        // Draw.
        self.draw_interval = (self.draw_interval + 1) % Self::DRAW_INTERVAL;
        if self.draw_interval == 0 {
            if self.need_redraw {
                if self.last_input > Self::IDLE_DELAY {
                    self.need_redraw = false;
                    log::info!("Disabled terminal to save cpu time. Press any key to re-enable.");
                }
                self.draw(performance);
            } else {
                if self.last_input <= Self::IDLE_DELAY {
                    self.need_redraw = true;
                }
            }
        }

        false
    }

    /// Return if we should quit.
    pub fn handle_inputs(&mut self, key: Key) -> bool {
        // Leave help mode.
        if self.help {
            self.help = false;
            return false;
        }

        // Handle keys based on selected tab.
        match self.current_tab {
            TerminalTab::Performance => {}
            TerminalTab::Log => match key {
                Key::Left => self.log_state.transition(&TuiWidgetEvent::LeftKey),
                Key::Right => self.log_state.transition(&TuiWidgetEvent::RightKey),
                Key::Up => self.log_state.transition(&TuiWidgetEvent::UpKey),
                Key::Down => self.log_state.transition(&TuiWidgetEvent::DownKey),
                Key::PageUp => self.log_state.transition(&TuiWidgetEvent::PrevPageKey),
                Key::PageDown => self.log_state.transition(&TuiWidgetEvent::NextPageKey),
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
                    return false;
                }
                _ => (),
            },
            TerminalTab::Exit => {
                if Key::Esc == key {
                    return true;
                }
            }
            TerminalTab::Info => {}
        }

        // Special keys need to be paired with return or they will be handled here too.
        if let Key::Char(c) = key {
            if c == '\t' {
                self.current_tab =
                    TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
            } else if c == '?' {
                self.help = true;
            }
        } else if Key::Esc == key {
            self.current_tab = TerminalTab::Exit;
        }

        false
    }

    fn draw(&mut self, performance: &PerformanceMetrics) {
        let _ = self.backend_terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                // .margin(5)
                .constraints([Constraint::Length(3), Constraint::Percentage(60), Constraint::Min(10)].as_ref())
                .split(size);

            // Draw tabs titles.
            let tab_titles = TerminalTab::TITLES.into_iter().map(|s| Spans::from(s)).collect();
            let mut tabs = Tabs::new(tab_titles)
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            // Select ? if we are in help mode.
            if self.help {
                tabs = tabs.select(TerminalTab::LEN + 1);
            } else {
                tabs = tabs.select(self.current_tab.into());
            }
            frame.render_widget(tabs, chunks[0]);

            if self.help {
                // Draw current tab in help mode.
                let text = match self.current_tab {
                    TerminalTab::Performance => vec![
                        Spans::from("Time is mesured in seconds."),
                        Spans::from("It does not factor in drawing the terminal (TODO)."),
                        Spans::from(format!("Budget is {}ms.", UPDATE_INTERVAL.as_millis())),
                    ],
                    TerminalTab::Log => vec![
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
                    ],
                    TerminalTab::Info => vec![
                        Spans::from("This is the help mode for the Info tab. Each tab has its own help mode."),
                        Spans::from("You can leave by pressing any key."),
                        Spans::from("Some keys apply to all tabs unless specifically prevented."),
                        Spans::from("| ?        | Go into help mode for the current tab."),
                        Spans::from("| tab      | Go to the next tab."),
                        Spans::from("| esc      | Shutdown server."),
                    ],
                    TerminalTab::Exit => vec![
                        Spans::from("You can shutdown the server here."),
                    ],
                };
                let paragraph = Paragraph::new(text)
                    .alignment(Alignment::Left)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, chunks[1]);
            } else {
                // Draw current tab.
                match self.current_tab {
                    TerminalTab::Performance => {
                        let data: Vec<(f64, f64)> = performance.recents.iter().zip((0i32..).into_iter()).map(|(y, i)| (i as f64, *y as f64)).collect();

                        let datasets = vec![
                            Dataset::default()
                                .name("Metascape total")
                                .marker(Marker::Braille)
                                .graph_type(GraphType::Line)
                                .style(Style::default().fg(Color::Cyan))
                                .data(&data),
                        ];

                        let x_label = ["0.0", "-5.0", "-10.0"].into_iter().map(Span::from).collect();
                        let y_label = ["0.0", "50.0", "100.0"].into_iter().map(Span::from).collect();

                        let chart = Chart::new(datasets)
                            .x_axis(Axis::default()
                                // .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                                .style(Style::default().fg(Color::White))
                                .bounds([0.0, performance.recents.len() as f64])
                                .labels(x_label))
                            .y_axis(Axis::default()
                                // .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                                .style(Style::default().fg(Color::White))
                                .bounds([0.0, PerformanceMetrics::BUDGET as f64])
                                .labels(y_label));

                            frame.render_widget(chart, chunks[1]);
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
                            Spans::from(format!("version: {}", common::VERSION)),
                            Spans::from(format!("port: {}", common::net::SERVER_PORT)),
                        ];
                        let paragraph = Paragraph::new(text)
                            .alignment(Alignment::Left)
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(paragraph, chunks[1]);
                    }
                    TerminalTab::Exit => {
                        let paragraph = Paragraph::new("Exit?");
                        frame.render_widget(paragraph, chunks[1]);
                    }
                };
            }

            // Draw logs.
            let mut log = TuiLoggerWidget::default()
                .style_error(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .style_warn(Style::default().fg(Color::Yellow))
                // .style_info(Style::default().fg(Color::Cyan))
                .style_debug(Style::default().fg(Color::Green))
                .style_trace(Style::default().fg(Color::Magenta))
                .block(Block::default().borders(Borders::ALL));
            log.state(&self.log_state);
            frame.render_widget(log, chunks[2]);
        });
    }
}
