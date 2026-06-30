use std::iter::Empty;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::exist::{self, RunSummary};
use crate::game::Game;

pub enum RunsMode {
    Normal,
    Runs(RunsPicker),
}

pub struct RunsPicker {
    feature: Vec<RunSummary>,
    state: ListState,
}

impl RunsPicker {
    pub fn open() -> Self {
        let feature = exist::list_runs();
        let mut state = ListState::default();
        if !feature.is_empty() { state.select(Some(0)); }
        Self { feature, state }
    }

    pub fn is_empty(&self) -> bool { self.feature.is_empty() }

    fn move_by(&mut self, delta: isize) {
        if self.feature.is_empty() { return; }
        let len = self.feature.len() as isize;
        let sel = self.state.selected().unwrap_or(0) as isize;
        let next = (sel + delta).rem_euclid(len) as usize;  // will wrap top<-->bottom
        self.state.select(Some(next));
    }
    pub fn go_up(&mut self)     { self.move_by(-1); }
    pub fn go_down(&mut self)   { self.move_by(1); }

    pub fn selected(&self) -> Option<&RunSummary> {
        self.state.selected().and_then(|i| self.feature.get(i))
    }

    pub fn accept(&self) -> Result<Game, String> {
        let Some(sum) = self.selected() else { return Err("none".into()); }
        if !sum.loadable {
            return Err("stale".into());
        }
        exist::load_run(&sum.path)
    }
}

pub fn render_picker(f: &mut Frame, area: Rect, picker: &mut RunsPicker) {
    let block = Block::default()
        .title("~runs~")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    if picker.is_empty() {
        let empty = List::new(vec![
            ListItem::new(
                Line::from(
                    Span::styled(
                        "content",
                        Style::default().fg(Color::Gray),
                )
            )
        )]).block(block);
        f.render_widget(empty, area);
        return;
    }
    let rows: Vec<ListItem> = picker.feature
        .iter()
        .map(|s| {
            let style = if s.loadable {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(Span::styled(format!(" {}", s.label), style)))
        }).colloct();

    let list = List::new(rows)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("-> ");
    f.render_stateful_widget(list, area, &mut picker.state);
}

// ===========================================================================
// INTEGRATION — three small edits in your App / event loop
// ===========================================================================
//
// 1) Give the App a mode field:
//        mode: Mode,                       // init to Mode::Normal
//
// 2) When the user types the `runs` command (in your command parser):
//        "runs" => { self.mode = Mode::Runs(RunPicker::open()); }
//    Delete the old `runs <n>` branch entirely — the picker replaces it.
//
// 3) In your key-event handler, branch on mode BEFORE your normal REPL keys:
//        match &mut self.mode {
//            Mode::Runs(p) => match key.code {
//                KeyCode::Up        => p.up(),
//                KeyCode::Down      => p.down(),
//                KeyCode::Esc       => self.mode = Mode::Normal,
//                KeyCode::Enter     => {
//                    match p.confirm() {
//                        Ok(game) => { self.game = game; self.push("Loaded run."); }
//                        Err(msg) => self.push(msg),
//                    }
//                    self.mode = Mode::Normal;
//                }
//                _ => {}
//            },
//            Mode::Normal => { /* your existing REPL input handling */ }
//        }
//
// 4) In your draw fn, after drawing the normal UI:
//        if let Mode::Runs(p) = &mut self.mode {
//            runs_picker::render(f, popup_rect(f.size()), p);
//        }
//    (popup_rect = a centered sub-rectangle; or just pass the full f.size().)
