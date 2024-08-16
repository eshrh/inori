use crate::model::selector_state::*;
use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn get_artist_list<'a>(model: &Model) -> List<'a> {
    let artists: Vec<String> = model
        .library
        .contents()
        .map(|artist| artist.name.clone())
        .collect();
    List::new(artists)
}

pub fn get_track_data<'a>(artist: &ArtistData) -> List<'a> {
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
}

pub fn render(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(50), Constraint::Min(3)])
        .split(frame.size());

    let artist_list = get_artist_list(model)
        .block(match model.library.active {
            LibActiveSelector::ArtistSelector => {
                Block::bordered().border_style(Color::Blue)
            }
            LibActiveSelector::TrackSelector => Block::bordered(),
        })
        .highlight_style(Color::Red);

    frame.render_stateful_widget(
        artist_list,
        layout[0],
        &mut model.library.artist_state,
    );

    let block_style = match model.library.active {
        LibActiveSelector::ArtistSelector => Block::bordered(),
        LibActiveSelector::TrackSelector => {
            Block::bordered().border_style(Color::Blue)
        }
    };
    if let Some(artist) = model.library.selected_item_mut() {
        let list = get_track_data(artist)
            .block(block_style)
            .highlight_style(Color::Red)
            .highlight_symbol(">");
        frame.render_stateful_widget(
            list,
            layout[1],
            &mut artist.track_sel_state,
        )
    }
}
