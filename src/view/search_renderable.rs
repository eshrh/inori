use super::search_doc::{
    build_alias_search_doc, has_match_in_span, has_match_in_spans, AliasSpan,
    SearchProjection,
};
use super::Theme;
use crate::model::title_alias::AliasMaps;
use crate::model::{AlbumData, ArtistData, InfoEntry};
use mpd::Song;
use ratatui::prelude::*;

pub trait SearchRenderable {
    fn base_text(&self) -> String;
    fn search_projection(&self) -> SearchProjection;
    fn base_style_at(&self, _char_idx: usize, _theme: &Theme) -> Style {
        Style::default()
    }
    fn album_alias_display(&self) -> Option<&str> {
        None
    }
    fn title_alias_display(&self) -> Option<&str> {
        None
    }
    fn album_alias_insert_at(&self) -> Option<usize> {
        None
    }
}

fn alias_spans(
    alias: &str,
    style: Style,
    search_span: AliasSpan,
    idxs: &[u32],
) -> Vec<Span<'static>> {
    let mut out = Vec::with_capacity(alias.chars().count() + 3);
    out.push(Span::from(" "));
    out.push(Span::from("[").style(style));
    for (k, c) in alias.chars().enumerate() {
        let mut span = Span::from(c.to_string()).style(style);
        if u32::try_from(search_span.start + k)
            .ok()
            .is_some_and(|j| idxs.contains(&j))
        {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
        out.push(span);
    }
    out.push(Span::from("]").style(style));
    out
}

pub fn render_searchable_line(
    item: &impl SearchRenderable,
    idxs: Option<&[u32]>,
    theme: &Theme,
) -> Line<'static> {
    let projection = item.search_projection();
    let mut out: Vec<Span> = item
        .base_text()
        .chars()
        .enumerate()
        .map(|(i, c)| {
            Span::from(c.to_string()).style(item.base_style_at(i, theme))
        })
        .collect();

    let Some(idxs) = idxs else {
        return Line::from(out);
    };

    for (i, span) in out.iter_mut().take(projection.base_len).enumerate() {
        if u32::try_from(i).ok().is_some_and(|j| idxs.contains(&j)) {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
    }

    if projection
        .album_alias
        .is_some_and(|s| has_match_in_span(idxs, s))
    {
        if let (Some(alias), Some(search_span)) =
            (item.album_alias_display(), projection.album_alias)
        {
            let spans =
                alias_spans(alias, theme.field_album, search_span, idxs);
            if let Some(insert_at) = item.album_alias_insert_at() {
                out.splice(insert_at..insert_at, spans);
            } else {
                out.extend(spans);
            }
        }
    }

    if projection
        .title_alias
        .is_some_and(|s| has_match_in_span(idxs, s))
    {
        if let (Some(alias), Some(search_span)) =
            (item.title_alias_display(), projection.title_alias)
        {
            out.extend(alias_spans(
                alias,
                theme.field_artistsort,
                search_span,
                idxs,
            ));
        }
    }

    if has_match_in_spans(idxs, &projection.variant_spans) {
        let mut marker =
            vec![Span::from("["), Span::from("VAR"), Span::from("] ")];
        for span in &mut marker {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
        out.splice(0..0, marker);
    }

    Line::from(out)
}

impl SearchRenderable for InfoEntry {
    fn base_text(&self) -> String {
        self.to_display_string()
    }
    fn search_projection(&self) -> SearchProjection {
        self.search_doc()
    }
    fn base_style_at(&self, char_idx: usize, theme: &Theme) -> Style {
        let mut cur = self.artist.chars().count();
        if let Some(artist_sort) = &self.artist_sort {
            if *artist_sort != self.artist {
                let len = artist_sort.chars().count();
                cur += 1;
                if char_idx >= cur && char_idx < cur + len + 2 {
                    return theme.field_artistsort;
                }
                cur += len + 2;
            }
        }
        if let Some(album) = &self.album {
            let len = album.chars().count();
            if char_idx == cur {
                return theme.slash_span;
            }
            cur += 1;
            if char_idx >= cur && char_idx < cur + len {
                return theme.field_album;
            }
            cur += len;
        }
        if self.title.is_some() && char_idx == cur {
            return theme.slash_span;
        }
        Style::default()
    }
    fn album_alias_display(&self) -> Option<&str> {
        self.album_alias.as_deref()
    }
    fn title_alias_display(&self) -> Option<&str> {
        self.title_alias.as_deref()
    }
    fn album_alias_insert_at(&self) -> Option<usize> {
        let mut cur = self.artist.chars().count();
        if let Some(artist_sort) = &self.artist_sort {
            if *artist_sort != self.artist {
                cur += 1 + artist_sort.chars().count() + 2;
            }
        }
        self.album.as_ref().map(|album| {
            cur += 1;
            cur + album.chars().count()
        })
    }
}

impl SearchRenderable for ArtistData {
    fn base_text(&self) -> String {
        self.to_fuzzy_find_str()
    }
    fn search_projection(&self) -> SearchProjection {
        build_alias_search_doc(&self.to_fuzzy_find_str(), None, None)
    }
    fn base_style_at(&self, char_idx: usize, theme: &Theme) -> Style {
        if char_idx >= self.name.chars().count() {
            theme.field_artistsort
        } else {
            Style::default()
        }
    }
}

impl SearchRenderable for AlbumData {
    fn base_text(&self) -> String {
        self.name.clone()
    }
    fn search_projection(&self) -> SearchProjection {
        build_alias_search_doc(
            &self.name,
            self.alias
                .as_deref()
                .map(|a| (a, self.alias_variants.as_slice())),
            None,
        )
    }
    fn album_alias_display(&self) -> Option<&str> {
        self.alias.as_deref()
    }
    fn album_alias_insert_at(&self) -> Option<usize> {
        Some(self.name.chars().count())
    }
}

pub struct QueueSearchRow<'a> {
    pub song: &'a Song,
}

impl SearchRenderable for QueueSearchRow<'_> {
    fn base_text(&self) -> String {
        self.song.title.clone().unwrap_or_default()
    }
    fn search_projection(&self) -> SearchProjection {
        build_alias_search_doc(
            &self.song.title.clone().unwrap_or_default(),
            None,
            None,
        )
    }
}

pub struct TrackSongSearchRow<'a> {
    pub song: &'a Song,
    pub aliases: &'a AliasMaps,
}

impl SearchRenderable for TrackSongSearchRow<'_> {
    fn base_text(&self) -> String {
        self.song
            .title
            .clone()
            .unwrap_or_else(|| "Unknown Song".to_string())
    }
    fn search_projection(&self) -> SearchProjection {
        let album = self.aliases.album_ref_for_song(self.song);
        let title = self.aliases.title_ref_for_path(&self.song.file);
        build_alias_search_doc(
            &self
                .song
                .title
                .clone()
                .unwrap_or_else(|| "Unknown Song".to_string()),
            album.map(|a| (a.alias, a.variants)),
            title.map(|a| (a.alias, a.variants)),
        )
    }
    fn album_alias_display(&self) -> Option<&str> {
        self.aliases.album_ref_for_song(self.song).map(|a| a.alias)
    }
    fn title_alias_display(&self) -> Option<&str> {
        self.aliases
            .title_ref_for_path(&self.song.file)
            .map(|a| a.alias)
    }
    fn album_alias_insert_at(&self) -> Option<usize> {
        Some(
            self.song
                .title
                .as_ref()
                .map_or("Unknown Song", String::as_str)
                .chars()
                .count(),
        )
    }
}
