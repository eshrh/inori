use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

pub fn render(model: &Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Max(3)])
        .split(frame.size());

    let artists: Vec<String> = model.library.contents.keys().cloned().collect();
    frame.render_widget(List::new(artists), layout[0]);
}
