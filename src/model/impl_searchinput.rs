use super::Filter;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

impl Filter {
    fn query_cursor(&self) -> usize {
        self.query_boundary(self.query_cursor.unwrap_or(self.query.len()))
    }

    fn query_boundary(&self, cursor: usize) -> usize {
        self.query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= cursor)
            .unwrap_or(self.query.len())
    }

    fn move_query_cursor(&mut self, cursor: usize) {
        let cursor = self.query_boundary(cursor);
        self.query_cursor = (cursor < self.query.len()).then_some(cursor);
    }

    pub fn edit_query(&mut self, key: KeyEvent) -> bool {
        use KeyCode::*;
        let cursor = self.query_cursor();
        let previous = || {
            self.query[..cursor]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(i, _)| i)
        };
        let next = || {
            self.query[cursor..]
                .graphemes(true)
                .next()
                .map_or(cursor, |g| cursor + g.len())
        };
        let (code, modifiers) = (key.code, key.modifiers);
        let control = modifiers == KeyModifiers::CONTROL;
        let plain = modifiers.is_empty() || modifiers == KeyModifiers::SHIFT;
        let range = match code {
            Home if plain => {
                self.move_query_cursor(0);
                return false;
            }
            End if plain => {
                self.move_query_cursor(self.query.len());
                return false;
            }
            Left if plain => {
                self.move_query_cursor(previous());
                return false;
            }
            Right if plain => {
                self.move_query_cursor(next());
                return false;
            }
            Char('a') if control => {
                self.move_query_cursor(0);
                return false;
            }
            Char('e') if control => {
                self.move_query_cursor(self.query.len());
                return false;
            }
            Char('b') if control => {
                self.move_query_cursor(previous());
                return false;
            }
            Char('f') if control => {
                self.move_query_cursor(next());
                return false;
            }
            Char('u') if control => 0..cursor,
            Char('k') if control => cursor..self.query.len(),
            Char('w') if control => {
                let mut start = cursor;
                let mut word = false;
                for (i, grapheme) in
                    self.query[..cursor].grapheme_indices(true).rev()
                {
                    let whitespace = grapheme.chars().all(char::is_whitespace);
                    if word && whitespace {
                        break;
                    }
                    word |= !whitespace;
                    start = i;
                }
                start..cursor
            }
            Backspace if plain => previous()..cursor,
            Delete if plain => cursor..next(),
            Char('d') if control => cursor..next(),
            Char(c) if plain && !c.is_control() => {
                self.query.insert(cursor, c);
                self.move_query_cursor(cursor + c.len_utf8());
                return true;
            }
            _ => return false,
        };
        if range.is_empty() {
            return false;
        }
        let cursor = range.start;
        self.query.replace_range(range, "");
        self.move_query_cursor(cursor);
        true
    }

    pub fn visible_query(&mut self, width: usize) -> (&str, usize) {
        if width == 0 {
            return ("", 0);
        }
        let cursor = self.query_cursor();
        let cursor_width = self.query[cursor..]
            .graphemes(true)
            .next()
            .map_or(0, |g| g.width());
        let available = width.saturating_sub(cursor_width.max(1));
        let mut start = self
            .query
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|&i| i <= self.query_offset.min(cursor))
            .last()
            .unwrap_or(0);
        if self.query[start..cursor].width() > available {
            start = cursor;
            let mut columns = 0;
            for (i, grapheme) in
                self.query[..cursor].grapheme_indices(true).rev()
            {
                columns += grapheme.width();
                if columns > available {
                    break;
                }
                start = i;
            }
        }
        let mut columns =
            self.query[start..].width() + usize::from(cursor_width == 0);
        for (i, grapheme) in self.query[..start].grapheme_indices(true).rev() {
            columns += grapheme.width();
            if columns > width {
                break;
            }
            start = i;
        }
        self.query_offset = start;
        let mut end = start;
        let mut columns = 0;
        for grapheme in self.query[start..].graphemes(true) {
            let next = columns + grapheme.width();
            if next > width {
                break;
            }
            columns = next;
            end += grapheme.len();
        }
        (&self.query[start..end], cursor - start)
    }
}
