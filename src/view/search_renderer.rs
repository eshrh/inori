use super::Theme;
use crate::model::Filter;
use ratatui::prelude::*;
use ratatui::widgets::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn render_search_box(
    filter: &mut Filter,
    active: bool,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let block = Block::bordered().border_type(BorderType::Thick);
    let inner = block.inner(area);
    let width = inner.width.saturating_sub(2);
    let (query, cursor) = filter.visible_query(width.into());
    let mut spans = vec![Span::from("> ")];
    if active && width > 0 {
        let (before, rest) = query.split_at(cursor);
        let character = rest.graphemes(true).next().unwrap_or("");
        let after = &rest[character.len()..];
        let character = if character.width() == 0 {
            "▏"
        } else {
            character
        };
        spans.extend([
            Span::styled(before, theme.search_query_active),
            Span::styled(
                character,
                theme.search_query_active.patch(theme.search_cursor),
            ),
            Span::styled(after, theme.search_query_active),
        ]);
    } else {
        spans.push(Span::styled(query, theme.search_query_inactive));
    }
    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}
