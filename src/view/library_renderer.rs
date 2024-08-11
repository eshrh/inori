use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

pub fn get_artist_list<'a>(model: &Model) -> List<'a> {
    let artists: Vec<String> = model.library.artists.iter().cloned().collect();
    List::new(artists).highlight_style(Style::default().fg(Color::Red))
}

pub fn get_track_data<'a>(model: &mut Model) -> List<'a> {
    let albums = match model.library.artist_selected() {
        Some(a) => a.albums.clone(),
        None => vec![],
    };
    List::new(albums)
}

pub fn render(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(50), Constraint::Min(3)])
        .split(frame.size());

    frame.render_stateful_widget(
        get_artist_list(model).block(Block::bordered()),
        layout[0],
        &mut model.library.artist_state,
    );

    let list = get_track_data(model);
    frame.render_stateful_widget(
        list,
        layout[1],
        &mut model.library.song_state,
    );
}
