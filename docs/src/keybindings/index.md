# Keybindings

You can see the keybindings by pressing the `?` key.

The default key bindings can be overridden.

## List of all default keybindings

#### Common

| Key                            | Description | Corresponding keybind |
| ------------------------------ | ----------- | --------------------- |
| <kbd>Ctrl-c</kbd> <kbd>q</kbd> | Quit app    | `force_quit` `quit`   |
| <kbd>?</kbd> <kbd>F1</kbd>     | Open help   | `help_toggle`         |

#### Commit List

| Key                                  | Description                                        | Corresponding keybind                        |
| ------------------------------------ | -------------------------------------------------- | -------------------------------------------- |
| <kbd>Down/Up</kbd> <kbd>j/k</kbd>    | Move down/up                                       | `navigate_down` `navigate_up`                |
| <kbd>J/K</kbd>                       | Move down/up                                       | `select_down` `select_up`                    |
| <kbd>Alt-Down</kbd> <kbd>Alt-j</kbd> | Move to parent commit                              | `go_to_parent`                               |
| <kbd>g/G</kbd>                       | Go to top/bottom                                   | `go_to_top` `go_to_bottom`                   |
| <kbd>Ctrl-f/b</kbd>                  | Scroll page down/up                                | `page_down` `page_up`                        |
| <kbd>Ctrl-d/u</kbd>                  | Scroll half page down/up                           | `half_page_down` `half_page_up`              |
| <kbd>Ctrl-e/y</kbd>                  | Scroll down/up                                     | `scroll_down` `scroll_up`                    |
| <kbd>H/M/L</kbd>                     | Select top/middle/bottom of the screen             | `select_top` `select_middle` `select_bottom` |
| <kbd>Enter</kbd>                     | Show commit details<br>Apply search (if searching) | `confirm`                                    |
| <kbd>Tab</kbd>                       | Open refs list                                     | `ref_list`                                   |
| <kbd>/</kbd>                         | Start search                                       | `search`                                     |
| <kbd>Esc</kbd>                       | Cancel search                                      | `cancel`                                     |
| <kbd>n/N</kbd>                       | Go to next/previous search match                   | `go_to_next` `go_to_previous`                |
| <kbd>Ctrl-t</kbd>                    | Toggle search target                               | `search_target_toggle`                       |
| <kbd>Ctrl-g</kbd>                    | Toggle ignore case                                 | `ignore_case_toggle`                         |
| <kbd>Ctrl-x</kbd>                    | Toggle fuzzy match                                 | `fuzzy_toggle`                               |
| <kbd>m</kbd>                         | Toggle commit mark                                 | `mark_toggle`                                |
| <kbd>R</kbd>                         | Refresh                                            | `refresh`                                    |
| <kbd>c/C</kbd>                       | Copy commit short/full hash                        | `short_copy` `full_copy`                     |
| <kbd>d</kbd>                         | Toggle custom user command view                    | `user_command_1`                             |

#### Git Commit Detail

Press Enter on a commit to open its changed-file tree and patch side by side.
The original commit detail implementation is retained separately.

| Key | Description | Corresponding keybind |
| --- | --- | --- |
| <kbd>Tab</kbd> / <kbd>Shift-Tab</kbd> | Toggle full-screen patch / file tree | `ref_list` / `focus_previous` |
| <kbd>Enter</kbd> | Expand/collapse the selected directory or open the selected file in the editor | `confirm` |
| <kbd>Esc</kbd> / <kbd>Backspace</kbd> | Return from full-screen patch to the file tree, or back to the commit list | `cancel` / `close` |
| <kbd>e</kbd> | Edit the selected working-tree file (file tree only) | `edit_file` |
| <kbd>/</kbd> | Search filenames in the file tree | `search` |
| <kbd>n/N</kbd> | Go to the next/previous matching file | `go_to_next` / `go_to_previous` |
| <kbd>j/k</kbd> / <kbd>Down/Up</kbd> | Select a file or scroll the patch | `navigate_down` / `navigate_up` |
| <kbd>h/l</kbd> / <kbd>Left/Right</kbd> | Collapse/expand a directory or scroll the patch horizontally | `navigate_left` / `navigate_right` |
| <kbd>Ctrl-f/b</kbd> / <kbd>PageDown/PageUp</kbd> | Move one page | `page_down` / `page_up` |
| <kbd>Ctrl-d/u</kbd> | Move half a page | `half_page_down` / `half_page_up` |
| <kbd>g/G</kbd> | Go to top/bottom | `go_to_top` / `go_to_bottom` |
| <kbd>J/K</kbd> | Select older/newer commit | `select_down` / `select_up` |
| <kbd>Alt-j</kbd> / <kbd>Alt-Down</kbd> | Select the first parent commit | `go_to_parent` |
| <kbd>R</kbd> | Refresh | `refresh` |
| <kbd>m</kbd> | Toggle commit mark | `mark_toggle` |
| <kbd>c/C</kbd> | Copy commit short/full hash | `short_copy` / `full_copy` |
| <kbd>d</kbd> | Open the custom user command view | `user_command_1` |

Patches compare against the first parent; root commits compare against an empty
tree. Patches larger than 2 MiB are truncated for display. Binary changes show
Git's binary-file summary.

Tab shows only the selected file's patch at full width. Enter and e have no
effect in this full-screen view. Tab or Esc restores the file tree.

In the file tree, / starts an incremental, case-insensitive substring search of
filenames only. The complete tree stays visible; matching filenames are
highlighted and their parent directories open automatically as you navigate to
them. Enter confirms the search without opening the editor. n/N cycles through
matches. While typing, Esc restores the selection, folding, and search that
were active before input began. After confirming, Esc clears the search before
another Esc returns to the commit list. Backspace edits the query while typing.
No matches leave the current selection intact and display a status message.

In the file tree, Enter on a directory toggles its children; Enter on a file
and e both open the working-tree file for editing. Its contents may differ from
the selected historical commit. Missing files, directories, symlinks, and bare
repositories report an error instead of creating a file. Set `VISUAL` or
`EDITOR` to a terminal editor such as `nvim` or `vim` (in that order of
precedence); the fallback is `vi`. Editor arguments are supported, for example
`EDITOR='nvim -n'`. Serie pauses while the editor runs in the same terminal,
then restores the selected file and panel when the editor exits.

#### Original Commit Detail

| Key                                  | Description                     | Corresponding keybind           |
| ------------------------------------ | ------------------------------- | ------------------------------- |
| <kbd>Esc</kbd> <kbd>Backspace</kbd>  | Close commit details            | `close` `cancel`                |
| <kbd>Down/Up</kbd> <kbd>j/k</kbd>    | Scroll down/up                  | `navigate_down` `navigate_up`   |
| <kbd>Ctrl-f/b</kbd>                  | Scroll page down/up             | `page_down` `page_up`           |
| <kbd>Ctrl-d/u</kbd>                  | Scroll half page down/up        | `half_page_down` `half_page_up` |
| <kbd>g/G</kbd>                       | Go to top/bottom                | `go_to_top` `go_to_bottom`      |
| <kbd>J/K</kbd>                       | Select older/newer commit       | `select_down` `select_up`       |
| <kbd>Alt-Down</kbd> <kbd>Alt-j</kbd> | Select parent commit            | `go_to_parent`                  |
| <kbd>m</kbd>                         | Toggle commit mark              | `mark_toggle`                   |
| <kbd>R</kbd>                         | Refresh                         | `refresh`                       |
| <kbd>c/C</kbd>                       | Copy commit short/full hash     | `short_copy` `full_copy`        |
| <kbd>d</kbd>                         | Toggle custom user command view | `user_command_1`                |

#### Refs List

| Key                                                | Description      | Corresponding keybind            |
| -------------------------------------------------- | ---------------- | -------------------------------- |
| <kbd>Esc</kbd> <kbd>Backspace</kbd> <kbd>Tab</kbd> | Close refs list  | `close` `cancel` `ref_list`      |
| <kbd>Down/Up</kbd> <kbd>j/k</kbd>                  | Move down/up     | `navigate_down` `navigate_up`    |
| <kbd>J/K</kbd>                                     | Move down/up     | `select_down` `select_up`        |
| <kbd>g/G</kbd>                                     | Go to top/bottom | `go_to_top` `go_to_bottom`       |
| <kbd>Right/Left</kbd> <kbd>l/h</kbd>               | Open/Close node  | `navigate_right` `navigate_left` |
| <kbd>R</kbd>                                       | Refresh          | `refresh`                        |
| <kbd>c</kbd>                                       | Copy ref name    | `short_copy`                     |

#### User Command

| Key                                  | Description                 | Corresponding keybind           |
| ------------------------------------ | --------------------------- | ------------------------------- |
| <kbd>Esc</kbd> <kbd>Backspace</kbd>  | Close user command          | `close` `cancel`                |
| <kbd>Down/Up</kbd> <kbd>j/k</kbd>    | Scroll down/up              | `navigate_down` `navigate_up`   |
| <kbd>J/K</kbd>                       | Scroll down/up              | `select_down` `select_up`       |
| <kbd>Ctrl-f/b</kbd>                  | Scroll page down/up         | `page_down` `page_up`           |
| <kbd>Ctrl-d/u</kbd>                  | Scroll half page down/up    | `half_page_down` `half_page_up` |
| <kbd>g/G</kbd>                       | Go to top/bottom            | `go_to_top` `go_to_bottom`      |
| <kbd>J/K</kbd>                       | Select older/newer commit   | `select_down` `select_up`       |
| <kbd>Alt-Down</kbd> <kbd>Alt-j</kbd> | Select parent commit        | `go_to_parent`                  |
| <kbd>m</kbd>                         | Toggle commit mark          | `mark_toggle`                   |
| <kbd>R</kbd>                         | Refresh                     | `refresh`                       |

#### Help

| Key                                                            | Description              | Corresponding keybind           |
| -------------------------------------------------------------- | ------------------------ | ------------------------------- |
| <kbd>Esc</kbd> <kbd>Backspace</kbd> <kbd>?</kbd> <kbd>F1</kbd> | Close help               | `close` `cancel` `help_toggle`  |
| <kbd>Down/Up</kbd> <kbd>j/k</kbd>                              | Scroll down/up           | `navigate_down` `navigate_up`   |
| <kbd>J/K</kbd>                                                 | Scroll down/up           | `select_down` `select_up`       |
| <kbd>Ctrl-f/b</kbd>                                            | Scroll page down/up      | `page_down` `page_up`           |
| <kbd>Ctrl-d/u</kbd>                                            | Scroll half page down/up | `half_page_down` `half_page_up` |
| <kbd>g/G</kbd>                                                 | Go to top/bottom         | `go_to_top` `go_to_bottom`      |

</details>

----

- [Custom Keybindings](./custom-keybindings.md)
