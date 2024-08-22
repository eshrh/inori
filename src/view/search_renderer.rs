use super::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn make_search_box<'a>(
    query: &'a String,
    active: bool,
    theme: &Theme,
) -> Paragraph<'a> {
    Paragraph::new(vec![Line::from(vec![
        Span::from("> "),
        Span::from(query).style(if active {
            theme.search_query_active
        } else {
            theme.search_query_inactive
        }),
    ])])
    .block(Block::bordered().border_type(BorderType::Thick))
}
