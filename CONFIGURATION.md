# Configuration

Config file is read from `$XDG_CONFIG_HOME/inori/config.toml`,
defaulting to `$HOME/.config/inori/config.toml` if it is not set.

## Keybindings

Keybindings set in the config file *override* the defaults if they are
set, but do not delete them.

Keybindings should be defined in a toml table called `keybindings` like
so:

``` toml
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
        one of
            - <space>
            - <tab>
            - <esc>
            - <backspace>
            - <delete>
            - <up>
            - <down>
            - <left>
            - <right>
            - <enter>

Each of the modifiers corresponds to a modifier key, `CTRL, META,
SUPER, CTRL+META`. So, your keybindings will look like "g g" or "C-c
C-n"

Here is the full list of commands, along with their defaults.

<table>
<thead>
<tr class="header">
<th>Command name</th>
<th>Explanation</th>
<th>Default key (dvorak)</th>
<th>Default key (qwerty)</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><code class="verbatim">up</code></td>
<td>move up</td>
<td>t</td>
<td>k</td>
</tr>
<tr class="even">
<td><code class="verbatim">down</code></td>
<td>move down</td>
<td>h</td>
<td>j</td>
</tr>
<tr class="odd">
<td><code class="verbatim">left</code></td>
<td>move left</td>
<td>d</td>
<td>h</td>
</tr>
<tr class="even">
<td><code class="verbatim">right</code></td>
<td>move right</td>
<td>n</td>
<td>l</td>
</tr>
<tr class="odd">
<td><code class="verbatim">toggle_playpause</code></td>
<td>toggles between play and pause</td>
<td>p</td>
<td>p</td>
</tr>
<tr class="even">
<td><code class="verbatim">select</code></td>
<td>act on the selected entry</td>
<td>&lt;enter&gt;</td>
<td>&lt;enter&gt;</td>
</tr>
<tr class="odd">
<td><code class="verbatim">quit</code></td>
<td>close the program</td>
<td>q</td>
<td>q</td>
</tr>
<tr class="even">
<td><code class="verbatim">switch_to_library</code></td>
<td>switch to library screen</td>
<td>1</td>
<td>1</td>
</tr>
<tr class="odd">
<td><code class="verbatim">switch_to_queue</code></td>
<td>switch to queue screen</td>
<td>2</td>
<td>2</td>
</tr>
<tr class="even">
<td><code class="verbatim">toggle_screen_lq</code></td>
<td>toggle between library/queue</td>
<td>&lt;tab&gt;</td>
<td>&lt;tab&gt;</td>
</tr>
<tr class="odd">
<td><code class="verbatim">toggle_panel</code></td>
<td>[library] switch between artist and track selector</td>
<td></td>
<td></td>
</tr>
<tr class="even">
<td><code class="verbatim">fold</code></td>
<td>[library/track] toggle fold album</td>
<td>&lt;space&gt;</td>
<td>&lt;space&gt;</td>
</tr>
<tr class="odd">
<td><code class="verbatim">clear_queue</code></td>
<td>clear queue</td>
<td>-</td>
<td>-</td>
</tr>
<tr class="even">
<td><code class="verbatim">local_search</code></td>
<td>search local selector</td>
<td>/</td>
<td>/</td>
</tr>
<tr class="odd">
<td><code class="verbatim">global_search</code></td>
<td>[library] global jumping search</td>
<td>g</td>
<td>C-g</td>
</tr>
<tr class="even">
<td><code class="verbatim">escape</code></td>
<td>escape</td>
<td>&lt;esc&gt;</td>
<td>&lt;esc&gt;</td>
</tr>
<tr class="odd">
<td><code class="verbatim">delete</code></td>
<td>[queue] deletes the selected item off queue</td>
<td>&lt;backspace&gt;</td>
<td>&lt;backspace&gt;</td>
</tr>
<tr class="even">
<td><code class="verbatim">toggle_repeat</code></td>
<td>toggle repeat</td>
<td>r</td>
<td>r</td>
</tr>
<tr class="odd">
<td><code class="verbatim">toggle_single</code></td>
<td>toggle single</td>
<td>s</td>
<td>s</td>
</tr>
<tr class="even">
<td><code class="verbatim">toggle_consume</code></td>
<td>toggle consume</td>
<td>c</td>
<td>c</td>
</tr>
<tr class="odd">
<td><code class="verbatim">toggle_random</code></td>
<td>toggle random</td>
<td>z</td>
<td>z</td>
</tr>
<tr class="even">
<td><code class="verbatim">top</code></td>
<td>jump to top</td>
<td>&lt;</td>
<td>g g</td>
</tr>
<tr class="odd">
<td><code class="verbatim">bottom</code></td>
<td>jump to bottom</td>
<td>&gt;</td>
<td>G</td>
</tr>
</tbody>
</table>

Note that you can specify multiple entries for each command, creating
multiple keybinds.

## Theme

Colors should be specified in a table called "theme", like this:

``` toml
[theme.item_to_color]
fg = COLOR
bg = COLOR
add_modifier = MODIFIERS
sub_modifier = MODIFIERS
```

All fields are optional. `COLOR` should be **a string** of either

rgb hex  
"#FF0000"

[ansi escape index](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit)  
"9"

ansi color code  
"White", "Red", "LightCyan", etc

`MODIFIERS` should be a string of "\<MODIFIER\>" joined by "|"
characters. The available modifiers are

- BOLD
- DIM
- ITALIC
- UNDERLINED
- SLOW<sub>BLINK</sub>
- RAPID<sub>BLINK</sub>
- REVERSED
- HIDDEN
- CROSSED<sub>OUT</sub>

For example, you might write `add_modifier = "BOLD | ITALIC"`.

Here is the full list of styles available for customization:

<table>
<thead>
<tr class="header">
<th>Name</th>
<th>Explanation</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><code class="verbatim">item_highlight_active</code></td>
<td>selected item in an active list</td>
</tr>
<tr class="even">
<td><code class="verbatim">item_highlight_inactive</code></td>
<td>selected item in an inactive list</td>
</tr>
<tr class="odd">
<td><code class="verbatim">block_active</code></td>
<td>active block border style</td>
</tr>
<tr class="even">
<td><code class="verbatim">status_artist</code></td>
<td>artist text in status</td>
</tr>
<tr class="odd">
<td><code class="verbatim">status_album</code></td>
<td>album text in status</td>
</tr>
<tr class="even">
<td><code class="verbatim">status_title</code></td>
<td>title text in status</td>
</tr>
<tr class="odd">
<td><code class="verbatim">artist_sort</code></td>
<td>albumartistsort field in fuzzy search displays</td>
</tr>
<tr class="even">
<td><code class="verbatim">album</code></td>
<td>generic album (track selection, queue)</td>
</tr>
<tr class="odd">
<td><code class="verbatim">playing</code></td>
<td>the "playing" text in status</td>
</tr>
<tr class="even">
<td><code class="verbatim">paused</code></td>
<td>the "paused" text in status</td>
</tr>
<tr class="odd">
<td><code class="verbatim">stopped</code></td>
<td>the "stopped" text in status</td>
</tr>
<tr class="even">
<td><code class="verbatim">slash_span</code></td>
<td>the slashes in global search</td>
</tr>
<tr class="odd">
<td><code class="verbatim">search_query_active</code></td>
<td>search query text when the search is active</td>
</tr>
<tr class="even">
<td><code class="verbatim">search_query_inactive</code></td>
<td>search query text when the search is inactive</td>
</tr>
</tbody>
</table>
