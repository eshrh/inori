use crate::backend::{
    self, Backend, Command, ReadRequest, Refresh, SubmitError,
};
use crate::event_handler::{EventHandler, Result};
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
    events: mpsc::Receiver<backend::Event>,
    input: EventHandler,
    loads: Loads,
    unacked: VecDeque<Option<QueueEdit>>,
}

#[derive(Default)]
struct Loads {
    generation: u64,
    artists: bool,
    tracks: HashSet<String>,
    global_search: bool,
    refresh: bool,
    idle: bool,
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
        let (backend, events) =
            Backend::start(model.config.mpd_address.clone())?;
        Ok(Self {
            model,
            backend,
            events,
            input: EventHandler::new(),
            loads: Loads::default(),
            unacked: VecDeque::new(),
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
        self.schedule_loads()?;
        self.draw(terminal)?;
        loop {
            let redraw = tokio::select! {
                event = self.input.next() => self.handle_input(event?)?,
                event = self.events.recv() => {
                    self.apply(event.ok_or("MPD workers stopped")?)?
                }
                _ = ticks.tick() => {
                    self.poll()?;
                    false
                }
            };
            if matches!(self.model.state, State::Done) {
                self.events.close();
                self.backend.shutdown();
                return Ok(());
            }
            update::update_screens(&mut self.model);
            self.schedule_loads()?;
            if redraw {
                self.draw(terminal)?;
            }
        }
    }

    fn handle_input(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                self.model.error = None;
                if let Some(action) = update::handle_key(&mut self.model, key) {
                    let edit = QueueEdit::from_command(&action.command);
                    match self.backend.try_command(action.command) {
                        Ok(()) => {}
                        Err(SubmitError::Full) => {
                            self.model.error =
                                Some(SubmitError::Full.to_string());
                            return Ok(true);
                        }
                        Err(error) => return Err(error.into()),
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

    fn poll(&mut self) -> Result<()> {
        if !self.loads.refresh {
            self.loads.refresh =
                submitted(self.backend.try_refresh(Refresh::all()))?;
        }
        if !self.loads.idle {
            self.loads.idle = submitted(self.backend.try_idle_poll())?;
        }
        Ok(())
    }

    fn schedule_loads(&mut self) -> Result<()> {
        if !matches!(self.model.screen, Screen::Library) {
            return Ok(());
        }
        let generation = self.loads.generation;
        if !self.model.library.loaded {
            if !self.loads.artists {
                self.loads.artists = submitted(
                    self.backend.try_read(ReadRequest::Artists { generation }),
                )?;
            }
            return Ok(());
        }
        if let Some(artist) = self.model.library.selected_item() {
            if !artist.fetched && !self.loads.tracks.contains(&artist.name) {
                let name = artist.name.clone();
                if submitted(self.backend.try_read(ReadRequest::Tracks {
                    generation,
                    artist: name.clone(),
                }))? {
                    self.loads.tracks.insert(name);
                }
            }
        }
        if self.model.library.global_search.entries.filter.active
            && !self.model.library.global_search.loaded
            && !self.loads.global_search
        {
            self.loads.global_search = submitted(
                self.backend
                    .try_read(ReadRequest::GlobalSearch { generation }),
            )?;
        }
        Ok(())
    }

    fn apply(&mut self, event: backend::Event) -> Result<bool> {
        match event {
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
                self.loads.idle = false;
                if changes.is_empty() {
                    return Ok(false);
                }
                if changes.contains(&Subsystem::Database) {
                    self.loads.generation += 1;
                    self.loads.artists = false;
                    self.loads.tracks.clear();
                    self.loads.global_search = false;
                    self.model.library.loaded = false;
                    self.model.library.global_search.loaded = false;
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

fn submitted(result: std::result::Result<(), SubmitError>) -> Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(SubmitError::Full) => Ok(false),
        Err(error) => Err(error.into()),
    }
}
