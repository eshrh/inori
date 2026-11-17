use mpd::client::StreamTypes;
use mpd::Client;
use std::env;
use std::fmt;
use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) struct SharedEndpoint {
    selected: Mutex<Option<Result<Address, String>>>,
    changed: Condvar,
    stopped: Arc<AtomicBool>,
}

impl SharedEndpoint {
    pub(super) fn new(stopped: Arc<AtomicBool>) -> Self {
        Self {
            selected: Mutex::new(None),
            changed: Condvar::new(),
            stopped,
        }
    }

    pub(super) fn select(
        &self,
        address: Option<&str>,
        control: &SocketControl,
    ) -> Result<Client<StreamTypes>, String> {
        let result = select(address, control);
        let selected = result
            .as_ref()
            .map(|(_, address)| address.clone())
            .map_err(Clone::clone);
        *self.selected.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(selected);
        self.changed.notify_all();
        result.map(|(client, _)| client)
    }

    pub(super) fn connect(
        &self,
        control: &SocketControl,
    ) -> Result<Client<StreamTypes>, String> {
        let mut selected =
            self.selected.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if self.stopped.load(Ordering::Acquire) {
                return Err("MPD worker stopped".into());
            }
            if let Some(selection) = &*selected {
                let address = selection.as_ref().map_err(Clone::clone)?.clone();
                drop(selected);
                return connect(&address, control);
            }
            selected = self
                .changed
                .wait(selected)
                .unwrap_or_else(|e| e.into_inner());
        }
    }

    pub(super) fn description(&self) -> String {
        match &*self.selected.lock().unwrap_or_else(|e| e.into_inner()) {
            Some(Ok(address)) => address.to_string(),
            _ => "unresolved endpoint".into(),
        }
    }

    pub(super) fn wake(&self) {
        let _selected = self.selected.lock().unwrap_or_else(|e| e.into_inner());
        self.changed.notify_all();
    }
}

pub(super) struct SocketControl {
    socket: Mutex<Option<StreamTypes>>,
    stopped: Arc<AtomicBool>,
}

impl SocketControl {
    pub(super) fn new(stopped: Arc<AtomicBool>) -> Self {
        Self {
            socket: Mutex::new(None),
            stopped,
        }
    }

    pub(super) fn close(&self) {
        let mut slot = self.socket.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(socket) = slot.take() {
            shutdown(&socket);
        }
    }

    fn register(&self, socket: &StreamTypes) -> io::Result<()> {
        let mut slot = self.socket.lock().unwrap_or_else(|e| e.into_inner());
        if self.stopped.load(Ordering::Acquire) {
            shutdown(socket);
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "MPD worker stopped",
            ));
        }
        let cloned = match socket {
            StreamTypes::Tcp(stream) => StreamTypes::Tcp(stream.try_clone()?),
            #[cfg(unix)]
            StreamTypes::Unix(stream) => StreamTypes::Unix(stream.try_clone()?),
        };
        *slot = Some(cloned);
        Ok(())
    }

    fn clear_timeouts(&self) -> io::Result<()> {
        let slot = self.socket.lock().unwrap_or_else(|e| e.into_inner());
        match slot.as_ref() {
            Some(StreamTypes::Tcp(stream)) => {
                stream.set_read_timeout(None)?;
                stream.set_write_timeout(None)
            }
            #[cfg(unix)]
            Some(StreamTypes::Unix(stream)) => {
                stream.set_read_timeout(None)?;
                stream.set_write_timeout(None)
            }
            None => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "MPD worker stopped",
            )),
        }
    }
}

#[derive(Clone)]
enum Address {
    Tcp(SocketAddr),
    #[cfg(unix)]
    Unix(PathBuf),
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(address) => address.fmt(f),
            #[cfg(unix)]
            Self::Unix(path) => path.display().fmt(f),
        }
    }
}

enum Candidate {
    Tcp(String),
    #[cfg(unix)]
    Unix(PathBuf),
}

fn select(
    explicit: Option<&str>,
    control: &SocketControl,
) -> Result<(Client<StreamTypes>, Address), String> {
    let mut last_error = "no MPD address available".to_string();
    for candidate in candidates(explicit) {
        let addresses: Vec<_> = match candidate {
            Candidate::Tcp(host) => match host.to_socket_addrs() {
                Ok(addresses) => addresses.map(Address::Tcp).collect(),
                Err(error) => {
                    last_error = format!("{host}: {error}");
                    continue;
                }
            },
            #[cfg(unix)]
            Candidate::Unix(path) => vec![Address::Unix(path)],
        };
        for address in addresses {
            if control.stopped.load(Ordering::Acquire) {
                return Err("MPD worker stopped".into());
            }
            match connect(&address, control) {
                Ok(client) => return Ok((client, address)),
                Err(error) => last_error = error,
            }
        }
    }
    Err(format!("failed to connect to MPD: {last_error}"))
}

fn connect(
    address: &Address,
    control: &SocketControl,
) -> Result<Client<StreamTypes>, String> {
    let result = (|| {
        let socket = match address {
            Address::Tcp(address) => {
                let stream =
                    TcpStream::connect_timeout(address, CONNECT_TIMEOUT)?;
                stream.set_read_timeout(Some(CONNECT_TIMEOUT))?;
                stream.set_write_timeout(Some(CONNECT_TIMEOUT))?;
                StreamTypes::Tcp(stream)
            }
            #[cfg(unix)]
            Address::Unix(path) => {
                let stream = UnixStream::connect(path)?;
                stream.set_read_timeout(Some(CONNECT_TIMEOUT))?;
                stream.set_write_timeout(Some(CONNECT_TIMEOUT))?;
                StreamTypes::Unix(stream)
            }
        };
        control.register(&socket)?;
        let client = Client::new(socket)?;
        control.clear_timeouts()?;
        Ok::<_, mpd::error::Error>(client)
    })();
    result.map_err(|error| {
        control.close();
        format!("{address}: {error}")
    })
}

fn candidates(explicit: Option<&str>) -> Vec<Candidate> {
    if let Some(address) = explicit {
        #[cfg(unix)]
        if address.starts_with('/') {
            return vec![Candidate::Unix(address.into())];
        }
        return vec![Candidate::Tcp(address.into())];
    }

    let mut candidates = Vec::new();
    if let Ok(host) = env::var("MPD_HOST") {
        if let Ok(port) = env::var("MPD_PORT") {
            candidates.push(Candidate::Tcp(format!("{host}:{port}")));
        }
        candidates.push(Candidate::Tcp(format!("{host}:6600")));
        #[cfg(unix)]
        if host.starts_with('/') {
            candidates.push(Candidate::Unix(host.into()));
        }
    }
    #[cfg(unix)]
    candidates.push(Candidate::Unix(
        env::var_os("XDG_RUNTIME_DIR")
            .map(|dir| PathBuf::from(dir).join("mpd/socket"))
            .unwrap_or_else(|| "/run/mpd/socket".into()),
    ));
    candidates.push(Candidate::Tcp("localhost:6600".into()));
    candidates
}

fn shutdown(socket: &StreamTypes) {
    let _ = match socket {
        StreamTypes::Tcp(stream) => stream.shutdown(Shutdown::Both),
        #[cfg(unix)]
        StreamTypes::Unix(stream) => stream.shutdown(Shutdown::Both),
    };
}
