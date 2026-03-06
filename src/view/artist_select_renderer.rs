use super::search_renderable::render_searchable_line;
use super::Theme;
use crate::model::proto::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn get_artist_list<'a>(model: &Model, theme: &Theme) -> List<'a> {
    if model.library.should_filter() {
        let indices = &model.library.artists.filter.cache.indices;
        List::new((0..model.library.display_len()).filter_map(|i| {
            let artist = model.library.display_get(i)?;
            let idxs_o = indices.get(i)?;
            Some(render_searchable_line(artist, Some(idxs_o), theme))
        }))
    } else {
        List::new(
            (0..model.library.storage_len())
                .filter_map(|i| model.library.storage_get(i))
                .map(|artist| artist.name.clone())
                .collect::<Vec<String>>(),
        )
    }
}

pub fn render_artist_list(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let artist_list = get_artist_list(model, theme)
        .block(
            match model.library.active {
                ArtistSelector => {
                    Block::bordered().border_style(theme.block_active)
                }
                TrackSelector => Block::bordered(),
            }
            .title("Artists"),
        )
        .highlight_style(match model.library.active {
            ArtistSelector => theme.item_highlight_active,
            TrackSelector => theme.item_highlight_inactive,
        });

    frame.render_stateful_widget(
        artist_list,
        area,
        &mut model.library.artist_state,
    );
}
