use mpd::AsyncClient;
use std::env;
use std::fmt;
use std::net::SocketAddr;
#[cfg(unix)]
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::net::{lookup_host, TcpStream};
use tokio::time::timeout;

pub(super) trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Stream for T {}
pub(super) type Client = AsyncClient<Box<dyn Stream>>;

#[derive(Clone)]
pub(super) enum Address {
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

impl Address {
    pub(super) async fn connect(&self) -> mpd::error::Result<Client> {
        timeout(Duration::from_secs(5), async {
            let stream: Box<dyn Stream> = match self {
                Self::Tcp(address) => {
                    Box::new(TcpStream::connect(address).await?)
                }
                #[cfg(unix)]
                Self::Unix(path) => Box::new(UnixStream::connect(path).await?),
            };
            AsyncClient::new(stream).await
        })
        .await
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "MPD connection timed out",
            )
        })?
    }
}

enum Candidate {
    Tcp(String),
    #[cfg(unix)]
    Unix(PathBuf),
}

pub(super) async fn select(
    explicit: Option<&str>,
) -> Result<(Client, Address), String> {
    let mut last_error = "no MPD address available".to_string();
    for candidate in candidates(explicit) {
        let addresses: Vec<Address> = match candidate {
            Candidate::Tcp(host) => match lookup_host(&host).await {
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
            match address.connect().await {
                Ok(client) => return Ok((client, address)),
                Err(error) => last_error = format!("{address}: {error}"),
            }
        }
    }
    Err(format!("failed to connect to MPD: {last_error}"))
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
