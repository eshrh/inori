#[derive(Copy, Clone)]
pub struct AliasSpan {
    pub start: usize,
    pub len: usize,
}

impl AliasSpan {
    pub fn contains(&self, idx: usize) -> bool {
        idx >= self.start && idx < self.start + self.len
    }
}

pub struct SearchProjection {
    pub text: String,
    pub base_len: usize,
    pub album_alias: Option<AliasSpan>,
    pub title_alias: Option<AliasSpan>,
    pub variant_spans: Vec<AliasSpan>,
}

pub fn build_alias_search_doc(
    base: &str,
    album_alias: Option<(&str, &[String])>,
    title_alias: Option<(&str, &[String])>,
) -> SearchProjection {
    let mut text = String::from(base);
    let base_len = base.chars().count();
    let mut cursor = base_len;
    let mut variant_spans = Vec::new();

    let album_span = album_alias.map(|(alias, variants)| {
        text.push(' ');
        cursor += 1;
        let span = AliasSpan {
            start: cursor,
            len: alias.chars().count(),
        };
        text.push_str(alias);
        cursor += span.len;
        for variant in variants {
            text.push(' ');
            cursor += 1;
            let var_span = AliasSpan {
                start: cursor,
                len: variant.chars().count(),
            };
            text.push_str(variant);
            cursor += var_span.len;
            variant_spans.push(var_span);
        }
        span
    });

    let title_span = title_alias.map(|(alias, variants)| {
        text.push(' ');
        cursor += 1;
        let span = AliasSpan {
            start: cursor,
            len: alias.chars().count(),
        };
        text.push_str(alias);
        cursor += span.len;
        for variant in variants {
            text.push(' ');
            cursor += 1;
            let var_span = AliasSpan {
                start: cursor,
                len: variant.chars().count(),
            };
            text.push_str(variant);
            cursor += var_span.len;
            variant_spans.push(var_span);
        }
        span
    });

    SearchProjection {
        text,
        base_len,
        album_alias: album_span,
        title_alias: title_span,
        variant_spans,
    }
}

pub fn has_match_in_span(indices: &[u32], span: AliasSpan) -> bool {
    indices
        .iter()
        .any(|&i| usize::try_from(i).ok().is_some_and(|j| span.contains(j)))
}

pub fn has_match_in_spans(indices: &[u32], spans: &[AliasSpan]) -> bool {
    spans.iter().any(|span| has_match_in_span(indices, *span))
}
