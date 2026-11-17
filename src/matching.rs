use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

pub const SYNC_LIMIT: usize = 2048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Global,
    Artists,
    Queue,
    Tracks(String),
}

pub struct Job {
    pub target: Target,
    pub query: String,
    pub haystacks: Arc<[Utf32String]>,
    pub top_k: Option<usize>,
}

pub struct Matched {
    pub target: Target,
    pub query: String,
    pub haystacks: Arc<[Utf32String]>,
    pub matches: Matches,
}

pub struct Matches {
    pub order: Vec<usize>,
    pub indices: Vec<Vec<u32>>,
    pub ranks: Vec<Option<usize>>,
}

pub fn rank(
    query: &str,
    haystacks: &[Utf32String],
    matcher: &mut Matcher,
    top_k: Option<usize>,
    cancelled: impl Fn() -> bool + Sync,
) -> Option<Matches> {
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let score = |start: usize,
                 strings: &[Utf32String],
                 matcher: &mut Matcher| {
        let mut scores = Vec::new();
        for (index, haystack) in strings.iter().enumerate() {
            if index % 1024 == 0 && cancelled() {
                return None;
            }
            if let Some(score) = pattern.score(haystack.slice(..), matcher) {
                if score > 0 {
                    scores.push((start + index, score));
                }
            }
        }
        Some(scores)
    };
    let mut scores = if haystacks.len() < 32_768 {
        score(0, haystacks, matcher)?
    } else {
        let workers = thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(4);
        let chunk_size = haystacks.len().div_ceil(workers);
        thread::scope(|scope| {
            let tasks: Vec<_> = haystacks
                .chunks(chunk_size)
                .enumerate()
                .map(|(chunk, strings)| {
                    let score = &score;
                    let mut local = matcher.clone();
                    scope.spawn(move || {
                        score(chunk * chunk_size, strings, &mut local)
                    })
                })
                .collect();
            tasks.into_iter().try_fold(Vec::new(), |mut scores, task| {
                scores.extend(task.join().unwrap()?);
                Some(scores)
            })
        })?
    };
    if cancelled() {
        return None;
    }
    scores.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    if cancelled() {
        return None;
    }
    let mut order = Vec::with_capacity(scores.len());
    let mut ranks = vec![None; haystacks.len()];
    let mut indices = Vec::new();
    for (rank, (index, _)) in scores.into_iter().enumerate() {
        if rank % 1024 == 0 && cancelled() {
            return None;
        }
        order.push(index);
        ranks[index] = Some(rank);
        if top_k.is_none_or(|limit| rank < limit) {
            let mut matched = Vec::new();
            pattern.indices(haystacks[index].slice(..), matcher, &mut matched);
            indices.push(matched);
        }
    }
    Some(Matches {
        order,
        indices,
        ranks,
    })
}

#[derive(Default)]
struct Pending {
    job: Option<(Job, u64)>,
    stopped: bool,
}

pub struct MatchingWorker {
    pending: Arc<(Mutex<Pending>, Condvar)>,
    ticket: Arc<AtomicU64>,
    results: mpsc::Receiver<Matched>,
    thread: Option<JoinHandle<()>>,
}

impl MatchingWorker {
    pub fn new(config: Config) -> io::Result<Self> {
        let pending =
            Arc::new((Mutex::new(Pending::default()), Condvar::new()));
        let ticket = Arc::new(AtomicU64::new(0));
        let (tx, results) = mpsc::channel(1);
        let worker_pending = pending.clone();
        let worker_ticket = ticket.clone();
        let thread = thread::Builder::new()
            .name("inori-matching".into())
            .spawn(move || {
                let mut matcher = Matcher::new(config);
                loop {
                    let (lock, ready) = &*worker_pending;
                    let mut pending = lock.lock().unwrap();
                    while pending.job.is_none() && !pending.stopped {
                        pending = ready.wait(pending).unwrap();
                    }
                    if pending.stopped {
                        break;
                    }
                    let (job, ticket) = pending.job.take().unwrap();
                    drop(pending);
                    let cancelled =
                        || worker_ticket.load(Ordering::Relaxed) != ticket;
                    let Some(matches) = rank(
                        &job.query,
                        &job.haystacks,
                        &mut matcher,
                        job.top_k,
                        cancelled,
                    ) else {
                        continue;
                    };
                    if !cancelled()
                        && tx
                            .blocking_send(Matched {
                                target: job.target,
                                query: job.query,
                                haystacks: job.haystacks,
                                matches,
                            })
                            .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            pending,
            ticket,
            results,
            thread: Some(thread),
        })
    }

    pub fn submit(&self, job: Job) {
        let ticket = self.ticket.fetch_add(1, Ordering::Relaxed) + 1;
        let (lock, ready) = &*self.pending;
        lock.lock().unwrap().job = Some((job, ticket));
        ready.notify_one();
    }

    pub fn cancel(&self) {
        self.ticket.fetch_add(1, Ordering::Relaxed);
        self.pending.0.lock().unwrap().job = None;
    }

    pub async fn next(&mut self) -> Option<Matched> {
        self.results.recv().await
    }
}

impl Drop for MatchingWorker {
    fn drop(&mut self) {
        self.results.close();
        self.cancel();
        self.pending.0.lock().unwrap().stopped = true;
        self.pending.1.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
