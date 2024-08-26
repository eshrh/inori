use super::Theme;
use crate::model::proto::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_str_with_idxs<'a>(
    str: String,
    idxs: &Vec<u32>,
    len: usize,
    theme: &Theme,
) -> Vec<Span<'a>> {
    let spans: Vec<Span> = str
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if idxs.contains(&u32::try_from(i).unwrap()) {
                Span::from(c.to_string())
                    .style(Style::default().add_modifier(Modifier::UNDERLINED))
            } else {
                Span::from(c.to_string())
            }
            .patch_style(if i >= len {
                theme.artist_sort
            } else {
                Style::default()
            })
        })
        .collect();
    spans
}

pub fn get_artist_list<'a>(model: &Model, theme: &Theme) -> List<'a> {
    if model.library.should_filter() {
        let indices = &model.library.artist_search.cache.indices;
        List::new(model.library.contents().zip(indices).map(
            |(artist, idxs_o)| {
                let len = artist.name.chars().count();
                let l = Line::from(render_str_with_idxs(
                    artist.to_fuzzy_find_str(),
                    idxs_o,
                    len,
                    theme,
                ));
                l
            },
        ))
    } else {
        List::new(
            model
                .library
                .contents()
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
