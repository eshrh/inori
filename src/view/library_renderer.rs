use crate::model::selector_state::*;
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

pub fn get_track_data<'a>(artist: &mut ArtistData) -> List<'a> {
    let albums = artist
        .contents()
        .iter()
        .map(|i| match i {
            TrackSelItem::Album(a) => a.name.clone(),
            TrackSelItem::Song(s) => {
                "    ".to_string() + &s.title.clone().unwrap()
            }
        })
        .collect::<Vec<String>>();

    List::new(albums)
        .block(Block::bordered())
        .highlight_style(Style::default().fg(Color::Red))
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

    match model.library.selected_item_mut() {
        Some(mut artist) => {
            let list = get_track_data(&mut artist);
            frame.render_stateful_widget(
                list,
                layout[1],
                &mut artist.track_sel_state,
            )
        }
        None => {}
    }
}
