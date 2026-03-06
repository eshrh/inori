# Configuration

The config file is read from `<config_dir>/inori/config.toml`.
On GNU/Linux and macOS, this is `$XDG_CONFIG_HOME/inori/config.toml` or `$HOME/.config/inori/config.toml`.
For other operating systems, check the
[platform-dirs documentation](https://docs.rs/platform-dirs/latest/platform_dirs/).

## General

- `mpd_address`
  - Type: String
  - Default: `localhost:6600`
  - The host and port to check for mpd. Alternatively you can set the
    `MPD_HOST` and `MPD_PORT` environment variables. Note that the
    configuration option currently has precedence.
- `seek_seconds`
  - Type: 64-bit integer
  - Default: 5
  - The time in seconds to seek by when using the `seek` and `seek_backwards` command
- `screens`
  - Type: Array of strings
  - Default: `["library", "queue"]`
  - Defines the screens mapped by keybindings. The first screen in the
    array is the initial startup screen. The last screen is the initial
    screen that `toggle_screen` (default: `<tab>`) toggles to.
  - The only two currently available screens are `"library"` and `"queue"`.
- `nucleo_prefer_prefix`
  - See [relevant nucleo docs](https://docs.rs/nucleo/latest/nucleo/struct.Config.html#structfield.prefer_prefix).
- `status_indicator_enabled`
  - Type: String
  - Default: `"#"`
  - Symbol shown when repeat/random/single/consume is enabled in the status bar.
- `status_indicator_disabled`
  - Type: String
  - Default: `"-"`
  - Symbol shown when repeat/random/single/consume is disabled in the status bar.

## Search Aliases

Search (global search, queue search, and library track search) supports
optional title and album aliases loaded from
`<config_dir>/inori/aliases.json`, which should contain a json array.

Each entry must contain exactly one of:
    - `path`: MPD `file` path
    - `album`: album name (note that this is *not* collision-safe)
And an `alias` key. Optionally, `variants` may be provided as a list of
alternative search terms.

While `alias` values are used as first class alternatives for the
canonical text, matching strings from `variants` are not displayed.
```json
[
  {
    "path": "Artist/アルバム/01 トラック.flac",
    "alias": "torakku",
    "variants": ["toraku"]
  },
  { "album": "づ", "alias": "zu", "variants": ["du"] }
]
```

### Alias generation

Aliases for libraries with mostly Japanese albums/tracks may be
generated with the included python script.

```bash
# reqs: [python-mpd2, fugashi[unidic-lite], pykakasi]
python3 -m pip install -r scripts/ja/requirements.txt
python3 scripts/ja/generate.py
```

- Be aware that unidic-lite takes ~250mb.
- Romanization style defaults to `kunrei`; `--style
  hepburn` is also supported, but is suboptimal for searching
  if you know Japanese.
- `--variant-style {hepburn,kunrei,passport}` may be provided multiple
  times to add multiple styles to the variants field.
- Use `--overwrite` to prefer newly generated aliases over existing
  entries.

PRs are welcome for alias generation scripts for other languages, or
more featureful scripts for Japanese. However, for tricky usecases,
like dialects or mixed Chinese-Japanese libraries, it's probably best
to use your own heuristics.

## Keybindings

### Keybinding sets

inori comes with sensible default keybindings for some commands. It
also includes two extra sets for convenience to hopefully suit most
users.

To enable the dvorak set, use

```toml
dvorak_keybindings = true
```

and likewise, to enable the qwerty set, use

```toml
qwerty_keybindings = true
```

Note that if both are set to true, the option set last will shadow the
option set first.

In general, the dvorak set will be more familiar to emacs users (this
is what I personally use), and the qwerty set will be familiar to vim
users.

### Keybinding syntax

Keybindings set in the config file _override_ the defaults if they are
set, but do not delete them.

Keybindings should be defined in a toml table called `keybindings` like
so:

```toml
[keybindings]
command1 = "KEYSTR1"
command2 = "KEYSTR2"
```

where a `"KEYSTR"` describes a keybinding with the following format that
will be reminiscent to emacs users:

```
KEYSTR := <KEYBIND> <KEYSTR> | <KEYBIND>
KEYBIND := <MODIFIER><CHARACTER>
MODIFIER := C- | M- | S- | C-M- | ""
CHARACTER := char | <SPECIAL_KEY>
SPECIAL_KEY := <space>
  | <tab>
  | <escape>
  | <backspace>
  | <delete>
  | <up>
  | <down>
  | <left>
  | <right>
  | <enter>
  | <home>
  | <end>
  | <pagedown>
  | <pageup>
```

Each of the modifiers corresponds to a modifier key, `CTRL, META,
SUPER, CTRL+META`. So, your keybindings will look like `g g` or `C-c
C-n` or `C-<space>`

You can create multiple keybinds for the same command using an array
of `KEYSTR`:

```toml
[keybindings]
command1 = ["KEYSTR1", "KEYSTR2"]
```

### List of commands and defaults

| Command name       | Explanation                                                          | default       | dvorak set | qwerty set |
|--------------------|----------------------------------------------------------------------|---------------|------------|------------|
| `up`               | move up                                                              | `<up>`, C-p   | t          | k          |
| `down`             | move down                                                            | `<down>`, C-n | h          | j          |
| `left`             | move left                                                            | `<left>`      | d          | h          |
| `right`            | move right                                                           | `<right>`     | n          | l          |
| `top`              | jump to top                                                          | `<home>`      | <          | g g        |
| `bottom`           | jump to bottom                                                       | `<end>`       | >          | G          |
| `screenful_up`     | scroll down one page, cursor to first line                           | `<pageup>`    | M-v        | C-b        |
| `screenful_down`   | scroll up one page, cursor to last line                              | `<pagedown>`  | C-v        | C-f        |
| `toggle_playpause` | toggles between play and pause                                       | p             |            |            |
| `next song`        | jumps to the next song in the queue                                  |               |            |            |
| `previous song`    | jumps to the previous song in the queue                              |               |            |            |
| `seek`             | seeks forward by `seek_seconds` (default: 5) seconds                 |               |            |            |
| `seek_backwards`   | seeks backwards by `seek_seconds` (default 5) seconds                |               |            |            |
| `select`           | act on the selected entry                                            | `<enter>`     |            |            |
| `select_and_next`  | act on the selected entry and then move down                         |               |            |            |
| `quit`             | close the program                                                    | q             |            |            |
| `screen_1`         | switch to screen 1 (default: library)                                | 1             |            |            |
| `screen_2`         | switch to screen 2 (default: queue)                                  | 2             |            |            |
| `toggle_screen`    | toggle between your last two used screens (default: library & queue) | `<tab>`       |            |            |
| `toggle_panel`     | [library] switch between artist and track selector                   |               |            |            |
| `fold`             | [library/track] toggle fold album                                    | `<space>`     |            |            |
| `clear_queue`      | clear queue                                                          | -             |            |            |
| `local_search`     | search local selector                                                | /             |            |            |
| `global_search`    | [library] global jumping search                                      | C-s           | g          | C-g        |
| `escape`           | escape                                                               | `<esc>`       | C-g        |            |
| `delete`           | [queue] deletes the selected item off queue                          | `<backspace>` |            |            |
| `toggle_repeat`    | toggle repeat                                                        | r             |            |            |
| `toggle_single`    | toggle single                                                        | s             |            |            |
| `toggle_consume`   | toggle consume                                                       | c             |            |            |
| `toggle_random`    | toggle random                                                        | z             |            |            |
| `update_db`        | update mpd db                                                        | u             |            |            |

Note that the dvorak/qwerty sets _do not_ delete the default
keybindings.

### Search keybinds

In any search field, `C-n` (down) and `C-p` (up) are always available for
navigation. In the global search, these are the *only* navigation
keys. `C-u` can be used to reset any search field without closing it.
`C-w` deletes the previous word in the active search query.

## Theme

Colors should be specified in a table called "theme", like this:

```toml
[theme.item_to_color]
fg = "<COLOR>"
bg = "<COLOR>"
add_modifier = ["<MODIFIER>"]
sub_modifier = ["<MODIFIER>"]
```

All fields are optional. `<COLOR>` should be one of

- rgb hex: `#FF0000`
- [ansi escape index](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit): `9`
- ansi color code: `White`, `Red`, `LightCyan`, etc

`<MODIFIER>` should be one of:

- BOLD
- DIM
- ITALIC
- UNDERLINED
- SLOW_BLINK
- RAPID_BLINK
- REVERSED
- HIDDEN
- CROSSED_OUT

For example, you might write `add_modifier = ["BOLD", "ITALIC"]`.

Here is the full list of styles available for customization:

| Name                      | Explanation                                    |
| ------------------------- | ---------------------------------------------- |
| `block_active`            | active block border style                      |
| `field_album`             | generic album (track selection, queue)         |
| `field_artistsort`        | albumartistsort field in fuzzy search displays |
| `item_highlight_active`   | selected item in an active list                |
| `item_highlight_inactive` | selected item in an inactive list              |
| `search_query_active`     | search query text when the search is active    |
| `search_query_inactive`   | search query text when the search is inactive  |
| `slash_span`              | the slashes in global search                   |
| `status_album`            | album text in status                           |
| `status_artist`           | artist text in status                          |
| `status_paused`           | the "paused" text in status                    |
| `status_playing`          | the "playing" text in status                   |
| `status_stopped`          | the "stopped" text in status                   |
| `status_title`            | title text in status                           |
