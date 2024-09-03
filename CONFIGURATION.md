# Configuration

Config file is read from `$XDG_CONFIG_HOME/inori/config.toml`,
defaulting to `$HOME/.config/inori/config.toml` if it is not set.

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
KEYSTR := <KEYBIND> <KEYSTR> | ""
KEYBIND := <MODIFIER><CHARACTER>
MODIFIER := C- | M- | S- | C-M- | ""
CHARACTER := char | <SPECIAL_KEY>
SPECIAL_KEY := <space>
  | <tab>
  | <esc>
  | <backspace>
  | <delete>
  | <up>
  | <down>
  | <left>
  | <right>
  | <enter>
  | <home>
  | <end>
```

Each of the modifiers corresponds to a modifier key, `CTRL, META,
SUPER, CTRL+META`. So, your keybindings will look like `g g` or `C-c
C-n` or `C-<space>`

Note that you can specify multiple entries for each command, creating
multiple keybinds.

### List of commands and defaults

| Command name        | Explanation                                        | default       | dvorak set | qwerty set |
|---------------------|----------------------------------------------------|---------------|------------|------------|
| `up`                | move up                                            | `<up>`        | t          | k          |
| `down`              | move down                                          | `<down>`      | h          | j          |
| `left`              | move left                                          | `<left>`      | d          | h          |
| `right`             | move right                                         | `<right>`     | n          | l          |
| `toggle_playpause`  | toggles between play and pause                     | p             |            |            |
| `select`            | act on the selected entry                          | `<enter>`     |            |            |
| `quit`              | close the program                                  | q             |            |            |
| `switch_to_library` | switch to library screen                           | 1             |            |            |
| `switch_to_queue`   | switch to queue screen                             | 2             |            |            |
| `toggle_screen_lq`  | toggle between library/queue                       | `<tab>`       |            |            |
| `toggle_panel`      | [library] switch between artist and track selector |               |            |            |
| `fold`              | [library/track] toggle fold album                  | `<space>`     |            |            |
| `clear_queue`       | clear queue                                        | -             |            |            |
| `local_search`      | search local selector                              | /             |            |            |
| `global_search`     | [library] global jumping search                    | C-s           | g          | C-g        |
| `escape`            | escape                                             | `<esc>`       | C-g        |            |
| `delete`            | [queue] deletes the selected item off queue        | `<backspace>` |            |            |
| `toggle_repeat`     | toggle repeat                                      | r             |            |            |
| `toggle_single`     | toggle single                                      | s             |            |            |
| `toggle_consume`    | toggle consume                                     | c             |            |            |
| `toggle_random`     | toggle random                                      | z             |            |            |
| `top`               | jump to top                                        | `<home>`   | <          | g g        |
| `bottom`            | jump to bottom                                     | `<end>` | >          | G          |

Note that the dvorak/qwerty sets *do not* delete the default
keybindings.

## Theme

Colors should be specified in a table called "theme", like this:

```toml
[theme.item_to_color]
fg = COLOR
bg = COLOR
add_modifier = MODIFIERS
sub_modifier = MODIFIERS
```

All fields are optional. `COLOR` should be **a string** of either

- rgb hex: "#FF0000"
- [ansi escape index](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit): "9"
- ansi color code: "White", "Red", "LightCyan", etc

`MODIFIERS` should be a string of "\<MODIFIER\>" joined by "|"
characters. The available modifiers are

- BOLD
- DIM
- ITALIC
- UNDERLINED
- SLOW_BLINK
- RAPID_BLINK
- REVERSED
- HIDDEN
- CROSSED_OUT

For example, you might write `add_modifier = "BOLD | ITALIC"`.

Here is the full list of styles available for customization:

| Name                      | Explanation                                    |
| ------------------------- | ---------------------------------------------- |
| `item_highlight_active`   | selected item in an active list                |
| `item_highlight_inactive` | selected item in an inactive list              |
| `block_active`            | active block border style                      |
| `status_artist`           | artist text in status                          |
| `status_album`            | album text in status                           |
| `status_title`            | title text in status                           |
| `artist_sort`             | albumartistsort field in fuzzy search displays |
| `album`                   | generic album (track selection, queue)         |
| `playing`                 | the "playing" text in status                   |
| `paused`                  | the "paused" text in status                    |
| `stopped`                 | the "stopped" text in status                   |
| `slash_span`              | the slashes in global search                   |
| `search_query_active`     | search query text when the search is active    |
| `search_query_inactive`   | search query text when the search is inactive  |
