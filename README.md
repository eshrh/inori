# inori

<p align="center">
  <span
    title="inori = i nor i; nor is sometimes denoted by X and i's
look like norm bars with major version as a p-norm"
  >
    <picture>
      <source
        media="(prefers-color-scheme: light)"
        srcset="https://github.com/eshrh/inori/raw/HEAD/images/inori-logo.svg"
      />
      <source
        media="(prefers-color-scheme: dark)"
        srcset="https://github.com/eshrh/inori/raw/HEAD/images/inori-logo-white.svg"
      />
      <img src="./images/inori-logo.svg" width="128px" alt="inori logo" />
    </picture>
  </span>
</p>

Client for the Music Player Daemon ([MPD](https://www.musicpd.org/)).

## Features

- Fuzzy search everywhere with
  [nucleo](https://github.com/helix-editor/nucleo)
  - Fully unicode aware, with special attention to the "albumartistsort"
    field
  - Global search across all tracks, albums, and artists
- Folding library interface inspired by [cmus](https://cmus.github.io/)
- Queue viewer and manipulation interface
- Configurable, chainable keybindings

## Usage/Installation

Run `cargo install inori`. inori is published on [crates.io](https://crates.io/crates/inori).

inori is also available on the AUR as [inori](https://aur.archlinux.org/packages/inori).
The PKGBUILD includes an option to switch between qwerty and dvorak defaults.

See [configuration.md](./CONFIGURATION.md) for config options, as well
as a full list of all default keybindings.

## Screenshots

![Screenshot showing the library view](./images/library.png)
![Screenshot showing the search feature](./images/search.png)
![Screenshot showing the queue view](./images/queue.png)

## Todo

- [ ] Playlist interface
- [ ] Compile feature flag for Japanese album/track title romanization for search using a tokenizer & dictionary
- [ ] More thorough customization options, especially for behavior & layout tweaks
- [ ] Spectrum visualizer like ncmpcpp

## Acknowledgements

- authors of [ratatui](https://ratatui.rs/) and
  [rust-mpd](https://docs.rs/mpd/latest/mpd/)
- [mmtc](https://github.com/figsoda/mmtc) and
  [rmptui](https://github.com/krolyxon/rmptui), two other rust mpd
  clients, helped me learn rust
- [@stephen-huan](https://github.com/stephen-huan): here from day one
