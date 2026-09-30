use crate::formats::Quality;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Gauge, List, ListState, Paragraph},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::Duration,
};

pub enum Msg {
    Percent(f64),
    Finished(Result<(), String>),
}

fn is_quit(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc)
        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
}

pub fn select_quality(
    title: &str,
    qualities: &[Quality],
    default: usize,
) -> anyhow::Result<Option<usize>> {
    let mut terminal = ratatui::init();
    let result = run_select(&mut terminal, title, qualities, default);
    ratatui::restore();
    result
}

fn run_select(
    terminal: &mut DefaultTerminal,
    title: &str,
    qualities: &[Quality],
    default: usize,
) -> anyhow::Result<Option<usize>> {
    let labels: Vec<String> = qualities.iter().map(|q| q.label()).collect();
    let mut state = ListState::default().with_selected(Some(default));

    loop {
        terminal.draw(|f| {
            let [head, body, foot] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .areas(f.area());

            f.render_widget(
                Paragraph::new(title).block(Block::bordered().title(" cf ")),
                head,
            );

            let list = List::new(labels.clone())
                .block(Block::bordered().title(" Quality "))
                .highlight_symbol("> ")
                .highlight_style(
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(list, body, &mut state);

            f.render_widget(
                Paragraph::new("up/down select   enter download   q quit")
                    .style(Style::new().fg(Color::DarkGray)),
                foot,
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if is_quit(key.code, key.modifiers) {
                return Ok(None);
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => state.select_next(),
                KeyCode::Enter => return Ok(state.selected()),
                _ => {}
            }
        }
    }
}

pub fn show_progress(
    title: &str,
    rx: Receiver<Msg>,
    cancel: &AtomicBool,
) -> anyhow::Result<Result<(), String>> {
    let mut terminal = ratatui::init();
    let result = run_progress(&mut terminal, title, rx, cancel);
    ratatui::restore();
    result
}

fn run_progress(
    terminal: &mut DefaultTerminal,
    title: &str,
    rx: Receiver<Msg>,
    cancel: &AtomicBool,
) -> anyhow::Result<Result<(), String>> {
    let mut percent = 0.0_f64;

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Percent(p) => percent = p,
                Msg::Finished(r) => return Ok(r),
            }
        }

        let cancelling = cancel.load(Ordering::Relaxed);

        terminal.draw(|f| {
            let [head, body, foot] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .areas(f.area());

            f.render_widget(
                Paragraph::new(title).block(Block::bordered().title(" cf ")),
                head,
            );

            let block_title = if cancelling {
                " Cancelling... "
            } else {
                " Downloading "
            };
            f.render_widget(
                Gauge::default()
                    .block(Block::bordered().title(block_title))
                    .gauge_style(Style::new().fg(Color::Cyan))
                    .use_unicode(true)
                    .ratio((percent / 100.0).clamp(0.0, 1.0)),
                body,
            );

            f.render_widget(
                Paragraph::new("q cancel").style(Style::new().fg(Color::DarkGray)),
                foot,
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && is_quit(key.code, key.modifiers) {
                    cancel.store(true, Ordering::Relaxed);
                }
            }
        }
    }
}
