use crate::model::selector_state::Selector;
use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn get_artist_list<'a>(model: &Model) -> List<'a> {
    let artists: Vec<String> = model
        .library
        .contents
        .iter()
        .map(|artist| artist.name.clone())
        .collect();
    List::new(artists).highlight_style(Style::default().fg(Color::Red))
}

pub fn get_track_data<'a>(model: &mut Model) -> List<'a> {
    let albums = match model.library.selected_item() {
        Some(a) => a.albums.iter().map(|a| a.name.clone()).collect(),
        None => vec![],
    };
    List::new(albums).block(Block::bordered())
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
    frame.render_widget(list, layout[1]);
}
