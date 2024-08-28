# Configuration

Config file is read from `$XDG_CONFIG_HOME/inori/config.toml`,
defaulting to `$HOME/.config/inori/config.toml` if it is not set.

## Keybindings

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

    KEYSTR := <KEYBIND> <KEYSTR> | ""
    KEYBIND := <MODIFIER><CHARACTER>
    MODIFIER := C- | M- | S- | C-M- | ""
    CHARACTER := char | <SPECIAL_KEY>
    SPECIAL_KEY :=
            | <space>
            | <tab>
            | <esc>
            | <backspace>
            | <delete>
            | <up>
            | <down>
            | <left>
            | <right>
            | <enter>

Each of the modifiers corresponds to a modifier key, `CTRL, META,
SUPER, CTRL+META`. So, your keybindings will look like `g g` or `C-c
C-n` or `C-<space>`

Here is the full list of commands, along with their defaults.

| Command name        | Explanation                                        | Default key (dvorak) | Default key (qwerty) |
| ------------------- | -------------------------------------------------- | -------------------- | -------------------- |
| `up`                | move up                                            | t                    | k                    |
| `down`              | move down                                          | h                    | j                    |
| `left`              | move left                                          | d                    | h                    |
| `right`             | move right                                         | n                    | l                    |
| `toggle_playpause`  | toggles between play and pause                     | p                    | p                    |
| `select`            | act on the selected entry                          | <enter>              | <enter>              |
| `quit`              | close the program                                  | q                    | q                    |
| `switch_to_library` | switch to library screen                           | 1                    | 1                    |
| `switch_to_queue`   | switch to queue screen                             | 2                    | 2                    |
| `toggle_screen_lq`  | toggle between library/queue                       | <tab>                | <tab>                |
| `toggle_panel`      | [library] switch between artist and track selector |                      |                      |
| `fold`              | [library/track] toggle fold album                  | <space>              | <space>              |
| `clear_queue`       | clear queue                                        | -                    | -                    |
| `local_search`      | search local selector                              | /                    | /                    |
| `global_search`     | [library] global jumping search                    | g                    | C-g                  |
| `escape`            | escape                                             | <esc>                | <esc>                |
| `delete`            | [queue] deletes the selected item off queue        | <backspace>          | <backspace>          |
| `toggle_repeat`     | toggle repeat                                      | r                    | r                    |
| `toggle_single`     | toggle single                                      | s                    | s                    |
| `toggle_consume`    | toggle consume                                     | c                    | c                    |
| `toggle_random`     | toggle random                                      | z                    | z                    |
| `top`               | jump to top                                        | <                    | g g                  |
| `bottom`            | jump to bottom                                     | >                    | G                    |

Note that you can specify multiple entries for each command, creating
multiple keybinds.

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
