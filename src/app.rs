use crate::backend::{
    self, Backend, Command, ReadRequest, Refresh, SubmitError,
};
use crate::event_handler::{EventHandler, Result};
use crate::matching::{MatchingWorker, Target};
use crate::model::proto::{Searchable, Selector};
use crate::model::{Model, Screen, State};
use crate::{update, view};
use mpd::{Id, Song, Subsystem};
use ratatui::backend::Backend as TerminalBackend;
use ratatui::crossterm::event::{Event, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};

pub struct App {
    model: Model,
    backend: Backend,
    session: Option<backend::Session>,
    events: mpsc::Receiver<backend::Event>,
    input: EventHandler,
    matching: MatchingWorker,
    searching: Option<(Target, String)>,
    loads: Loads,
    unacked: VecDeque<Option<QueueEdit>>,
    want: Refresh,
    update_db_on_startup: bool,
}

#[derive(Default)]
struct Loads {
    generation: u64,
    artists: bool,
    tracks: HashSet<String>,
    global_search: bool,
    refresh: bool,
}

#[derive(Clone, Copy)]
enum QueueEdit {
    Delete(Id),
    Swap(Id, Id),
    Clear,
}

impl QueueEdit {
    fn from_command(command: &Command) -> Option<Self> {
        match command {
            Command::Delete(id) => Some(Self::Delete(*id)),
            Command::Swap(from, to) => Some(Self::Swap(*from, *to)),
            Command::Clear => Some(Self::Clear),
            _ => None,
        }
    }

    fn apply(self, songs: &mut Vec<Song>) {
        match self {
            Self::Delete(id) => {
                songs.retain(|song| !song.place.is_some_and(|p| p.id == id))
            }
            Self::Swap(from, to) => {
                let from = songs
                    .iter()
                    .position(|song| song.place.is_some_and(|p| p.id == from));
                let to = songs
                    .iter()
                    .position(|song| song.place.is_some_and(|p| p.id == to));
                if let Some((from, to)) = from.zip(to) {
                    songs.swap(from, to);
                }
            }
            Self::Clear => songs.clear(),
        }
        for (index, song) in songs.iter_mut().enumerate() {
            if let Some(place) = &mut song.place {
                place.pos = index as u32;
            }
        }
    }
}

impl App {
    pub fn new(model: Model) -> Result<Self> {
        let mut config = nucleo_matcher::Config::DEFAULT;
        config.prefer_prefix = model.config.nucleo_prefer_prefix;
        let matching = MatchingWorker::new(config)?;
        let (backend, events) =
            Backend::start(model.config.mpd_address.clone());
        let update_db_on_startup = model.config.update_db_on_startup;
        Ok(Self {
            model,
            backend,
            session: None,
            events,
            input: EventHandler::new(),
            matching,
            searching: None,
            loads: Loads::default(),
            unacked: VecDeque::new(),
            want: Refresh::empty(),
            update_db_on_startup,
        })
    }

    pub async fn run<B: TerminalBackend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B::Error: 'static,
    {
        let mut ticks = time::interval(Duration::from_millis(500));
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        self.schedule_loads();
        self.draw(terminal)?;
        loop {
            let redraw = tokio::select! {
                event = self.input.next() => self.handle_input(event?)?,
                event = self.events.recv() => {
                    self.apply(event.ok_or("MPD workers stopped")?)?
                }
                matched = self.matching.next() => {
                    let matched = matched.ok_or("search worker stopped")?;
                    if self.searching.as_ref().is_some_and(|(target, query)| {
                        *target == matched.target && *query == matched.query
                    }) {
                        self.searching = None;
                    }
                    self.model.apply_match(matched)
                }
                _ = ticks.tick() => {
                    self.want |= Refresh::STATUS;
                    false
                }
            };
            if matches!(self.model.state, State::Done) {
                self.events.close();
                self.backend.shutdown();
                return Ok(());
            }
            update::update_screens(&mut self.model);
            self.schedule_loads();
            self.schedule_search();
            if redraw {
                self.draw(terminal)?;
            }
        }
    }

    fn handle_input(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                self.model.error = None;
                if update::search_boundary(key) {
                    self.matching.cancel();
                    self.searching = None;
                }
                if let Some(action) = update::handle_key(&mut self.model, key) {
                    let edit = QueueEdit::from_command(&action.command);
                    match self
                        .session
                        .as_ref()
                        .ok_or(SubmitError::Closed)
                        .and_then(|session| session.try_command(action.command))
                    {
                        Ok(()) => {}
                        Err(error) => {
                            self.model.error = Some(error.to_string());
                            return Ok(true);
                        }
                    }
                    if let Some(message) = action.on_accept {
                        let next = update::handle_msg(&mut self.model, message);
                        debug_assert!(next.is_none());
                    }
                    self.unacked.push_back(edit);
                    if let Some(edit) = edit {
                        if matches!(edit, QueueEdit::Delete(_)) {
                            let next =
                                self.model.queue.selected().and_then(|index| {
                                    index.checked_sub(1).or_else(|| {
                                        (self.model.queue.display_len() > 1)
                                            .then_some(1)
                                    })
                                });
                            self.model.queue.set_selected(next);
                        }
                        let mut songs = self.model.queue.songs.items.clone();
                        edit.apply(&mut songs);
                        self.replace_queue(songs);
                    }
                }
            }
            Event::Resize(width, height) => {
                self.model.frame_size = Rect::new(0, 0, width, height)
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn schedule_loads(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        if !self.loads.refresh
            && !self.want.is_empty()
            && session.try_refresh(self.want).is_ok()
        {
            self.loads.refresh = true;
            self.want = Refresh::empty();
        }
        if !matches!(self.model.screen, Screen::Library) {
            return;
        }
        let generation = self.loads.generation;
        if !self.model.library.loaded {
            if !self.loads.artists {
                self.loads.artists = session
                    .try_read(ReadRequest::Artists { generation })
                    .is_ok();
            }
            return;
        }
        if let Some(artist) = self.model.library.selected_item() {
            if !artist.fetched && !self.loads.tracks.contains(&artist.name) {
                let name = artist.name.clone();
                if session
                    .try_read(ReadRequest::Tracks {
                        generation,
                        artist: name.clone(),
                    })
                    .is_ok()
                {
                    self.loads.tracks.insert(name);
                }
            }
        }
        if self.model.library.global_search.entries.filter.active
            && !self.model.library.global_search.loaded
            && !self.loads.global_search
        {
            self.loads.global_search = session
                .try_read(ReadRequest::GlobalSearch { generation })
                .is_ok();
        }
    }

    fn schedule_search(&mut self) {
        let job = self.model.pending_search();
        let target = job
            .as_ref()
            .map(|job| (job.target.clone(), job.query.clone()));
        if self.searching != target {
            self.searching = target;
            if let Some(job) = job {
                self.matching.submit(job);
            } else {
                self.matching.cancel();
            }
        }
    }

    fn apply(&mut self, event: backend::Event) -> Result<bool> {
        match event {
            backend::Event::Connected { session, snapshot } => {
                self.apply_snapshot(*snapshot);
                self.model.error = None;
                if self.update_db_on_startup {
                    match session.try_command(Command::UpdateDatabase) {
                        Ok(()) => {
                            self.update_db_on_startup = false;
                            self.unacked.push_back(None);
                        }
                        Err(error) => {
                            self.model.error = Some(error.to_string());
                        }
                    }
                }
                self.session = Some(session);
                self.loads.refresh = false;
                self.invalidate_library();
            }
            backend::Event::Disconnected(error) => {
                self.session = None;
                self.unacked.clear();
                self.loads.refresh = false;
                self.want = Refresh::empty();
                self.model.error = Some(error);
                self.invalidate_library();
            }
            backend::Event::Completed { snapshot, error } => {
                let pending = self.unacked.pop_front();
                debug_assert!(pending.is_some());
                if error.is_some() {
                    self.model.error = error;
                }
                self.apply_snapshot(*snapshot);
            }
            backend::Event::Refreshed(snapshot) => {
                self.loads.refresh = false;
                self.apply_snapshot(*snapshot);
            }
            backend::Event::Artists {
                generation,
                artists,
            } if generation == self.loads.generation => {
                self.loads.artists = false;
                update::build_library::apply_artists(&mut self.model, artists);
            }
            backend::Event::Tracks {
                generation,
                artist,
                songs,
            } if generation == self.loads.generation => {
                self.loads.tracks.remove(&artist);
                update::build_library::apply_tracks(
                    &mut self.model,
                    &artist,
                    songs,
                );
            }
            backend::Event::GlobalSearch { generation, songs }
                if generation == self.loads.generation =>
            {
                self.loads.global_search = false;
                self.model.apply_global_search_contents(songs);
            }
            backend::Event::Idle(changes) => {
                if changes.is_empty() {
                    return Ok(false);
                }
                if changes.contains(&Subsystem::Database) {
                    self.invalidate_library();
                }
                if changes.contains(&Subsystem::Queue) {
                    self.want |= Refresh::all();
                }
                if changes.contains(&Subsystem::Player) {
                    self.want |= Refresh::STATUS | Refresh::CURRENT_SONG;
                }
                if changes.contains(&Subsystem::Options)
                    || changes.contains(&Subsystem::Update)
                {
                    self.want |= Refresh::STATUS;
                }
                if changes.contains(&Subsystem::Database)
                    || changes.contains(&Subsystem::Update)
                {
                    self.model.reload_aliases()?;
                }
            }
            backend::Event::Error(error) => return Err(error.into()),
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn invalidate_library(&mut self) {
        self.loads.generation += 1;
        self.loads.artists = false;
        self.loads.tracks.clear();
        self.loads.global_search = false;
        self.model.library.loaded = false;
        self.model.library.global_search.loaded = false;
    }

    fn apply_snapshot(&mut self, snapshot: backend::Snapshot) {
        if let Some(status) = snapshot.status {
            self.model.status = status;
        }
        if let Some(current_song) = snapshot.current_song {
            self.model.currentsong = current_song;
        }
        if let Some(mut songs) = snapshot.queue {
            for edit in self.unacked.iter().flatten() {
                edit.apply(&mut songs);
            }
            self.replace_queue(songs);
        }
    }

    fn replace_queue(&mut self, songs: Vec<Song>) {
        self.model.queue.replace_songs(
            songs,
            &mut self.model.matcher,
            self.model.frame_size.height.into(),
        );
    }

    fn draw<B: TerminalBackend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B::Error: 'static,
    {
        terminal.draw(|frame| {
            self.model.frame_size = frame.area();
            view::view(&mut self.model, frame);
        })?;
        Ok(())
    }
}
