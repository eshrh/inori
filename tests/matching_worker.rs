#![allow(dead_code)]

include!("../src/main.rs");

use matching::{rank, Job, MatchingWorker, Target};
use nucleo_matcher::{Config, Matcher, Utf32String};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn literal_queries_stable_ties_and_inverse_ranks_are_preserved() {
    let strings: Vec<Utf32String> = [
        "BetaExtra",
        "Beta",
        r"\gap!",
        r"!gap\",
        "日本語 haru no uta",
    ]
    .into_iter()
    .map(Utf32String::from)
    .collect();
    let mut matcher = Matcher::new(Config::DEFAULT);
    let matches = rank("Beta", &strings, &mut matcher, None, || false).unwrap();
    assert_eq!(matches.order[..2], [0, 1]);
    assert_eq!(matches.ranks[..2], [Some(0), Some(1)]);
    let matches = rank(r"\!", &strings, &mut matcher, None, || false).unwrap();
    assert_eq!(matches.order[0], 2);
    assert_eq!(matches.order.len(), 1);
    let matches = rank("haru", &strings, &mut matcher, None, || false).unwrap();
    assert_eq!(matches.order[0], 4);
    assert!(!matches.indices[0].is_empty());
}

#[tokio::test]
async fn a_new_query_supersedes_pending_work() {
    let mut worker = MatchingWorker::new(Config::DEFAULT).unwrap();
    let haystacks: Arc<[Utf32String]> = (0..100_000)
        .map(|index| {
            Utf32String::from(format!("Beta {index} 春の歌 haru no uta"))
        })
        .collect::<Vec<_>>()
        .into();
    for query in ["b", "be", "bet", "never_matches_xyz"] {
        worker.submit(Job {
            target: Target::Global,
            query: query.into(),
            haystacks: haystacks.clone(),
            top_k: Some(30),
        });
    }
    let matched = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let result = worker.next().await.unwrap();
            if result.query == "never_matches_xyz" {
                break result;
            }
        }
    })
    .await
    .unwrap();
    assert!(Arc::ptr_eq(&matched.haystacks, &haystacks));
    assert!(matched.matches.order.is_empty());
}

#[test]
fn dropping_a_worker_closes_its_full_result_channel_before_joining() {
    let worker = MatchingWorker::new(Config::DEFAULT).unwrap();
    let haystacks: Arc<[Utf32String]> = vec!["Beta".into()].into();
    for query in ["b", "be"] {
        worker.submit(Job {
            target: Target::Global,
            query: query.into(),
            haystacks: haystacks.clone(),
            top_k: None,
        });
        std::thread::sleep(Duration::from_millis(20));
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        drop(worker);
        tx.send(()).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
}
