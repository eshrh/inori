use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use nucleo_matcher::{Utf32Str, Utf32String};

pub fn compute_orders(
    query: &str,
    strings: &Vec<Utf32String>,
    matcher: &mut Matcher,
) -> Vec<Option<usize>> {
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let scores = strings
        .iter()
        .map(|i| pattern.score(i.slice(..), matcher))
        .collect::<Vec<Option<u32>>>();
    let mut order = scores
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, Option<u32>)>>();
    order.sort_by(|a, b| b.1.unwrap_or(0).cmp(&a.1.unwrap_or(0)));
    let order = order
        .iter()
        .map(|i| {
            if i.1.is_some_and(|score| score > 0) {
                Some(i.0)
            } else {
                None
            }
        })
        .collect::<Vec<Option<usize>>>();
    order
}

pub fn compute_indices(
    query: &str,
    strings: Vec<&Utf32String>,
    matcher: &mut Matcher,
) -> Vec<Option<Vec<u32>>> {
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut indices: Vec<Option<Vec<u32>>> = Vec::new();
    for s in strings {
        let mut tmp: Vec<u32> = Vec::new();
        pattern.indices(s.slice(..), matcher, &mut tmp);
        indices.push(Some(tmp));
    }
    indices
}
