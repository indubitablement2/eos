use common::{factions::Factions, *};
use crossbeam::channel::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{
    io::{self, stdin, Stdout},
    thread::spawn,
    time::{Duration, Instant},
};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Sparkline, Tabs},
};
use tui_logger::{TuiLoggerTargetWidget, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

use crate::Metascape;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, FromPrimitive,
)]
#[repr(usize)]
enum TerminalTab {
    Performance,
    Log,
    Factions,
    #[default]
    Info,
}
impl TerminalTab {
    const LEN: usize = TerminalTab::Info as usize + 1;
    const TITLES: [&'static str; TerminalTab::LEN + 1] =
        ["Performance", "Log", "Factions", "Info", "?"];
}
impl Default for TerminalTab {
    fn default() -> Self {
        Self::Info
    }
}

/// Metrics in µs.
pub struct PerformanceMetrics {
    last_used: u64,

    max_used_lifetime: u64,
    num_over_budged: u64,

    used_recents: [u64; PerformanceMetrics::NUM_RECENT as usize],
    max_used_recent: u64,
    average_used_recent: u64,
}
impl PerformanceMetrics {
    const MAX_EXPECTED_USED: u64 = UPDATE_INTERVAL.as_micros() as u64;
    const NUM_RECENT: u64 = 78;

    fn update_basic(&mut self, last_used: u64) {
        // Update current.
        self.last_used = last_used;

        // Update maxs.
        self.max_used_lifetime = self.max_used_lifetime.max(last_used);
        if last_used > PerformanceMetrics::MAX_EXPECTED_USED {
            self.num_over_budged += 1;
        }
    }

    fn update_recents(&mut self) {
        self.used_recents
            .copy_within(0..PerformanceMetrics::NUM_RECENT as usize - 1, 1);
        *self.used_recents.first_mut().unwrap() = self.last_used;

        // Update recent statictics.
        let mut sum_used_recent = 0;
        self.max_used_recent = 0;
        self.used_recents.iter().for_each(|used| {
            sum_used_recent += *used;
            self.max_used_recent = self.max_used_recent.max(*used);
        });
        self.average_used_recent = sum_used_recent / PerformanceMetrics::NUM_RECENT;
    }
}
impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            last_used: Default::default(),
            max_used_lifetime: Default::default(),
            num_over_budged: Default::default(),
            used_recents: [0; PerformanceMetrics::NUM_RECENT as usize],
            max_used_recent: Default::default(),
            average_used_recent: Default::default(),
        }
    }
}

pub struct Terminal {
    backend_terminal: tui::Terminal<TermionBackend<RawTerminal<Stdout>>>,
    input_receiver: Receiver<Key>,

    current_tab: TerminalTab,
    /// Display help for current tab and disable terminal to save cpu usage.
    help: bool,
    /// If idle, automaticaly go to help mode.
    last_input: Instant,
    need_redraw: bool,

    log_state: TuiWidgetState,

    performance_metascape: PerformanceMetrics,
    performance_terminal: PerformanceMetrics,
    performance_total: PerformanceMetrics,

    faction_list_widget_state: ListState,
    faction_edit: bool,
    faction_edit_widget_state: ListState,
    faction_edit_target_increment: isize,
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
            current_tab: TerminalTab::default(),
            help: false,
            last_input: Instant::now(),
            need_redraw: true,
            log_state: TuiWidgetState::default(),
            performance_metascape: PerformanceMetrics::default(),
            performance_terminal: PerformanceMetrics::default(),
            performance_total: PerformanceMetrics::default(),
            faction_list_widget_state: ListState::default(),
            faction_edit: false,
            faction_edit_widget_state: ListState::default(),
            faction_edit_target_increment: 10,
        })
    }

    pub fn update(&mut self, stop_main: &mut bool, metascape: &mut Metascape) {
        let mut received_input = false;
        // Handle inputs.
        while let Ok(key) = self.input_receiver.try_recv() {
            received_input = true;

            self.need_redraw = true;
            // Check if we are in help mode.
            if self.help == true {
                self.help = false;
                continue;
            }

            // Handle keys based on selected tab.
            match self.current_tab {
                TerminalTab::Performance => {
                    if let Key::Char(c) = key {
                        if c == 'r' {
                            self.performance_metascape = PerformanceMetrics::default();
                            self.performance_terminal = PerformanceMetrics::default();
                            self.performance_total = PerformanceMetrics::default();
                        }
                    }
                }
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
                        continue;
                    }
                    _ => (),
                },
                TerminalTab::Factions => {
                    if self.faction_edit {
                        match key {
                            Key::Up => self.faction_edit_widget_state.select(Some(
                                self.faction_edit_widget_state
                                    .selected()
                                    .unwrap_or(0)
                                    .checked_sub(1)
                                    .unwrap_or(1),
                            )),
                            Key::Down => self.faction_edit_widget_state.select(Some(
                                (self.faction_edit_widget_state.selected().unwrap_or(1) + 1) % 2,
                            )),
                            Key::Esc => {
                                self.faction_edit = false;
                                self.faction_edit_widget_state.select(None);
                                continue;
                            }
                            Key::Char(c) => {
                                if c == '\n' {
                                    let selected_faction = if let Some(s) =
                                        self.faction_list_widget_state.selected()
                                    {
                                        s
                                    } else {
                                        self.faction_edit = false;
                                        continue;
                                    };
                                    if let Some(selected) =
                                        self.faction_edit_widget_state.selected()
                                    {
                                        let mut faction = &mut metascape
                                            .world
                                            .get_resource_mut::<Factions>()
                                            .unwrap()
                                            .factions[selected_faction];
                                        match selected {
                                            0 => {
                                                faction.disabled = !faction.disabled;
                                                info!(
                                                    "Disabled = {} for faction {}",
                                                    faction.disabled, selected_faction
                                                );
                                            }
                                            1 => {
                                                faction.target_colonies =
                                                    faction.target_colonies.saturating_add_signed(
                                                        self.faction_edit_target_increment,
                                                    );
                                                info!(
                                                    "Set faction target colony to {}",
                                                    faction.target_colonies
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Key::Right => {
                                if let Some(selected) = self.faction_edit_widget_state.selected() {
                                    match selected {
                                        1 => self.faction_edit_target_increment += 10,
                                        _ => {}
                                    }
                                } else {
                                    self.faction_edit = false;
                                }
                            }
                            Key::Left => {
                                if self.faction_edit_widget_state.selected() == Some(1) {
                                    self.faction_edit_target_increment -= 10;
                                } else {
                                    self.faction_edit = false;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key {
                            Key::Up => self.faction_list_widget_state.select(Some(
                                self.faction_list_widget_state
                                    .selected()
                                    .unwrap_or(0)
                                    .checked_sub(1)
                                    .unwrap_or(31),
                            )),
                            Key::Down => self.faction_list_widget_state.select(Some(
                                (self.faction_list_widget_state.selected().unwrap_or(31) + 1) % 32,
                            )),
                            Key::Right => self.faction_edit = true,
                            _ => (),
                        }
                    }
                }
                TerminalTab::Info => {}
            }

            // Special keys need to be paired with continue or they will be handled here too.
            if let Key::Char(c) = key {
                if c == '\t' {
                    self.current_tab =
                        TerminalTab::from((usize::from(self.current_tab) + 1) % TerminalTab::LEN);
                } else if c == '?' {
                    self.help = true;
                }
            } else if Key::Esc == key {
                // TODO: Ask to confirm.
                *stop_main = true;
                break;
            }
        }

        // Detect idle.
        if received_input {
            self.last_input = Instant::now();
        } else if self.last_input.elapsed() > Duration::from_secs(120) {
            self.help = true;
        }

        // Don't redraw if we don't need to (idle in help mode).
        if !self.need_redraw {
            return;
        }

        // Draw.
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
                tabs = tabs.select(TerminalTab::LEN);
            } else {
                tabs = tabs.select(self.current_tab.into());
            }
            frame.render_widget(tabs, chunks[0]);

            if self.help {
                self.need_redraw = false;

                // Draw current tab in help mode.
                let text = match self.current_tab {
                    TerminalTab::Performance => vec![
                        Spans::from("Time is mesured in ms (milliseconds)."),
                        Spans::from(format!("Budget is {}ms.", PerformanceMetrics::MAX_EXPECTED_USED / 1000)),
                        Spans::from("| r        | Reset performance metrics."),
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
                    TerminalTab::Factions => vec![
                        Spans::from("This is the help mode for the Info tab. Each tab has its own help mode."),
                        Spans::from("You can leave by pressing any key."),
                        Spans::from("In help mode, the terminal will not redraw to save cpu time."),
                        Spans::from("Some keys apply to all tabs unless specifically prevented."),
                        Spans::from("| ?        | Go into help mode for the current tab."),
                        Spans::from("| tab      | Go to the next tab."),
                        Spans::from("| esc      | Shutdown server."),
                    ],
                    TerminalTab::Info => vec![
                        Spans::from("This is the help mode for the Info tab. Each tab has its own help mode."),
                        Spans::from("You can leave by pressing any key."),
                        Spans::from("In help mode, the terminal will not redraw to save cpu time."),
                        Spans::from("Some keys apply to all tabs unless specifically prevented."),
                        Spans::from("| ?        | Go into help mode for the current tab."),
                        Spans::from("| tab      | Go to the next tab."),
                        Spans::from("| esc      | Shutdown server."),
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
                        let inner_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Ratio(1, 3),
                                Constraint::Ratio(1, 3),
                                Constraint::Ratio(1, 3),
                            ])
                            .split(chunks[1].inner(&Margin {
                                vertical: 0,
                                horizontal: 0,
                            }));

                        let sparkline_total = Sparkline::default()
                            .block(
                                Block::default()
                                    .title(format!(
                                        "Total|lifetime max{:6.2}|over {}|now:{:6.2}|max:{:6.2}|avg:{:6.2}",
                                        self.performance_total.max_used_lifetime as f32 / 1000.0,
                                        self.performance_total.num_over_budged,
                                        self.performance_total.last_used as f32 / 1000.0,
                                        self.performance_total.max_used_recent as f32 / 1000.0,
                                        self.performance_total.average_used_recent as f32 / 1000.0,
                                    ))
                                    .borders(Borders::LEFT | Borders::RIGHT),
                            )
                            .data(&self.performance_total.used_recents)
                            .style(Style::default().fg(Color::Yellow));
                        frame.render_widget(sparkline_total, inner_chunks[0]);

                        let sparkline_metascape = Sparkline::default()
                            .block(
                                Block::default()
                                    .title(format!(
                                        "Metascape|lifetime max{:6.2}|over {}|now:{:6.2}|max:{:6.2}|avg:{:6.2}",
                                        self.performance_metascape.max_used_lifetime as f32 / 1000.0,
                                        self.performance_metascape.num_over_budged,
                                        self.performance_metascape.last_used as f32 / 1000.0,
                                        self.performance_metascape.max_used_recent as f32 / 1000.0,
                                        self.performance_metascape.average_used_recent as f32 / 1000.0,
                                    ))
                                    .borders(Borders::LEFT | Borders::RIGHT),
                            )
                            .data(&self.performance_metascape.used_recents)
                            .style(Style::default().fg(Color::Yellow));
                        frame.render_widget(sparkline_metascape, inner_chunks[1]);

                        let sparkline_terminal = Sparkline::default()
                            .block(
                                Block::default()
                                    .title(format!(
                                        "Terminal|lifetime max{:6.2}|over {}|now:{:6.2}|max:{:6.2}|avg:{:6.2}",
                                        self.performance_terminal.max_used_lifetime as f32 / 1000.0,
                                        self.performance_terminal.num_over_budged,
                                        self.performance_terminal.last_used as f32 / 1000.0,
                                        self.performance_terminal.max_used_recent as f32 / 1000.0,
                                        self.performance_terminal.average_used_recent as f32 / 1000.0,
                                    ))
                                    .borders(Borders::LEFT | Borders::RIGHT),
                            )
                            .data(&self.performance_terminal.used_recents)
                            .style(Style::default().fg(Color::Yellow));
                        frame.render_widget(sparkline_terminal, inner_chunks[2]);
                    }
                    TerminalTab::Log => {
                        let log = TuiLoggerTargetWidget::default()
                            .state(&self.log_state)
                            .highlight_style(Style::default().fg(Color::Yellow))
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(log, chunks[1]);
                    }
                    TerminalTab::Factions => {
                        let mut factions = metascape.world.get_resource_mut::<Factions>().unwrap();

                        if self.faction_edit {
                            if let Some(selected) = self.faction_list_widget_state.selected() {
                                let faction = &mut factions.factions[selected];

                                let items = [
                                    if faction.disabled {
                                        ListItem::new(format!("Disabled: {}", faction.disabled)).style(Style::default().add_modifier(Modifier::DIM))
                                    } else {
                                        ListItem::new(format!("Disabled: {}", faction.disabled))
                                    },
                                    ListItem::new(format!("Target number of colonies: {} (increment <- ({}) ->)", faction.target_colonies, self.faction_edit_target_increment)),
                                ];

                                let list = List::new(items).highlight_symbol(">>");
                                frame.render_stateful_widget(list, chunks[1], &mut self.faction_edit_widget_state);
                            } else {
                                self.faction_edit = false;
                            }
                        } else {
                            let items: Vec<ListItem> = factions
                                .factions
                                .iter()
                                .zip(0u8..)
                                .map(|(faction, id)| {
                                    let mut item =  ListItem::new(format!("({}) {} - {} colonies / {} target", id, faction.name, faction.colonies.len(), faction.target_colonies));
                                    if faction.disabled {
                                        item = item.style(Style::default().add_modifier(Modifier::DIM))
                                    }
                                    item
                                })
                                .collect();
                            let list = List::new(items).highlight_symbol(">>");
                            frame.render_stateful_widget(list, chunks[1], &mut self.faction_list_widget_state);
                        }
                    }
                    TerminalTab::Info => {
                        let text = vec![
                            Spans::from(format!("version: {}", Version::CURRENT)),
                            Spans::from(format!("port: {}", common::SERVER_PORT)),
                        ];
                        let paragraph = Paragraph::new(text)
                            .alignment(Alignment::Left)
                            .block(Block::default().borders(Borders::ALL));
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

    pub fn update_performance_metrics(&mut self, total: u64, metascape: u64, terminal: u64) {
        self.performance_total.update_basic(total);
        self.performance_metascape.update_basic(metascape);
        self.performance_terminal.update_basic(terminal);
        // Do not waste cpu time updating recents if we are not looking at it.
        if self.current_tab == TerminalTab::Performance.into() {
            self.performance_total.update_recents();
            self.performance_metascape.update_recents();
            self.performance_terminal.update_recents();
        }
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
