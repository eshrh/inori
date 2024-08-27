# inori

Client for the Music Player Daemon ([MPD](https://www.musicpd.org/))

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

Run `cargo install --path .`

The default keybindings use dvorak-convenient movement keys (`dhtn`).
Build/install with the command line args
`--features qwerty_movement_keys --no-default-features` for qwerty-vim
style movement.

See [configuration.md](./CONFIGURATION.md) for config options, as well
as a full list of all default keybindings.

## Screenshots

![](./images/library.png) ![](./images/search.png)
![](./images/queue.png)

## Acknowledgements

- authors of [ratatui](https://ratatui.rs/) and
  [rust-mpd](https://docs.rs/mpd/latest/mpd/)
- [mmtc](https://github.com/figsoda/mmtc) and
  [rmptui](https://github.com/krolyxon/rmptui), two other rust mpd
  clients, helped me learn rust
- @stephen-huan: here from day one
