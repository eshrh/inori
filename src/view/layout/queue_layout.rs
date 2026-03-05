use crate::model::*;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;

use super::InoriLayout;

#[derive(Default)]
pub struct QueueLayout {
    pub header: Rect,
    pub search: Option<Rect>,
    pub queue: Rect,
    pub progress: Rect,
}

impl InoriLayout for QueueLayout {
    fn new(frame_rect: Rect, model: &Model) -> Self {
        let mut new = QueueLayout::default();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Max(4), Min(1), Max(3)])
            .split(frame_rect);
        let content = Layout::vertical(vec![Max(3), Min(1)]).split(layout[1]);
        new.header = layout[0];

        if model.queue.songs.filter.active {
            new.search = Some(content[0]);
            new.queue = content[1];
        } else {
            new.queue = layout[1];
        }
        new.progress = layout[2];
        new
    }
}
