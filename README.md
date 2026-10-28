# Serie

[![Crate Status](https://img.shields.io/crates/v/serie.svg)](https://crates.io/crates/serie)
[![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=fff&labelColor=000&color=fff)](https://ratatui.rs)

A rich git commit graph in your terminal, like magic 📚

<img src="./img/demo.gif">

(This demo shows [Ratatui](https://github.com/ratatui/ratatui) repository!)

## About

Serie ([`/zéːriə/`](https://lusingander.github.io/serie/faq/index.html#how-do-i-pronounce-serie)) is a TUI application that uses the terminal emulators' image display protocol to render commit graphs like `git log --graph --all`.

### Why?

While some users prefer to use Git via CLI, they often rely on a GUI or feature-rich TUI to view commit logs. Others may find `git log --graph` sufficient.

Personally, I found the output from `git log --graph` difficult to read, even with additional options. Learning complex tools just to view logs seemed cumbersome.

### Goals

- Provide a rich `git log --graph` experience in the terminal.
- Offer commit graph-centric browsing of Git repositories.

### Non-Goals

- Implement a fully-featured Git client.
- Create a TUI application with a complex UI.
- Works in any terminal environment.

## Documentation

For detailed usage, configuration, and advanced features, see [the full documentation](https://lusingander.github.io/serie/).

## Requirements

- Git
- Supported terminal emulator
  - Refer to [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html) for details.

## Installation

For this fork, install the `serie-timo` formula after its first release:

```sh
brew install Timo-In/tap/serie-timo
```

It provides the `serie` command and conflicts with the upstream `serie` formula.
Fork releases are available on [Timo-In/serie](https://github.com/Timo-In/serie/releases).
The following Cargo command installs the upstream version from crates.io.

If you're using Cargo:

```
$ cargo install --locked serie
```

For other download options, see [Installation](https://lusingander.github.io/serie/getting-started/installation.html).

## Maintaining the Timo fork

`timo/dev` is the development branch; `timo/main` is the default and stable
release branch. Merge changes between these published branches instead of
rebasing their shared history. To integrate an upstream update, fetch
`lusingander/serie` as an `upstream` remote, merge `upstream/master` into
`timo/dev`, and let CI pass before merging into `timo/main`.

The independent [Timo CI](.github/workflows/timo-ci.yml) checks both branches
and their pull requests. Its required status check is named **`cargo test`**;
that check passes only when all platform checks and release-tool tests pass.
The author workflows remain unchanged. In this fork's Actions settings,
disable **Build**, **Deny**, **Pinact**, **Deploy GitHub Pages**, and **Release**,
and keep **Timo CI** and **Timo Release** enabled. In particular, disable the
inherited Release workflow before pushing a fork release tag, as both release
workflows otherwise match that tag. If workflows are not listed yet, enable
Actions for the fork and push the workflow commit first.

Once GitHub lists the workflows, the inherited ones can also be disabled with:

```sh
for workflow in build.yml deny.yml pinact.yml gh-pages.yml release.yml; do
  gh workflow disable "$workflow" --repo Timo-In/serie
done
```

To release:

1. Set the package version in `Cargo.toml` and `Cargo.lock` to a fork version,
   such as `0.10.1-timo`. All fork release versions end in `-timo`; use a new
   version for each release instead of replacing an existing tag.
2. Push `timo/dev` and merge its pull request into `timo/main` with **Create a
   merge commit** after `cargo test` passes. Fetch the updated stable branch locally.
3. Create and push an annotated tag matching the package version:

   ```sh
   git switch timo/main
   git pull --ff-only origin timo/main
   git tag -a v0.10.1-timo -m "release: v0.10.1-timo"
   git push origin v0.10.1-timo
   ```

[Timo Release](.github/workflows/timo-release.yml) rejects a tag whose version
does not match Cargo or whose commit is not contained in `timo/main`. It runs
CI again, builds macOS and Linux binaries for ARM64 and x86-64, and publishes
the archives, `SHA256SUMS`, and generated `serie-timo.rb` to GitHub Releases.
The Linux builds use Ubuntu 22.04 (glibc). The Homebrew job validates the
formula and tests a macOS ARM64 installation before committing
`Formula/serie-timo.rb` to `Timo-In/homebrew-tap` using the repository secret
`HOMEBREW_TAP_GITHUB_TOKEN`. The token needs permission to write that tap's
contents. A failed Homebrew job can be retried after correcting the token or
tap configuration; the GitHub Release remains available.

## Usage

### Basic

Run `serie` in the directory where your git repository exists.

```
$ cd <your git repository>
$ serie
```

### Options

```
Serie - A rich git commit graph in your terminal, like magic 📚

Usage: serie [OPTIONS]

Options:
  -n, --max-count <NUMBER>        Maximum number of commits to render
  -p, --protocol <TYPE>           Image protocol to render graph [default: auto] [possible values: auto, iterm, kitty, kitty-unicode]
  -o, --order <TYPE>              Commit ordering algorithm [default: chrono] [possible values: chrono, topo]
  -g, --graph-width <TYPE>        Commit graph image cell width [default: auto] [possible values: auto, double, single]
  -s, --graph-style <TYPE>        Commit graph image edge style [default: rounded] [possible values: rounded, angular, curved]
      --graph-antialias           Enable antialiasing for commit graph images
      --no-graph-antialias        Disable antialiasing for commit graph images
  -i, --initial-selection <TYPE>  Initial selection of commit [default: latest] [possible values: latest, head]
  -b, --primary-branch <BRANCH>   Primary branch to keep on the leftmost column
  -h, --help                      Print help
  -V, --version                   Print version
```

For details on each option, see [Command Line Options](https://lusingander.github.io/serie/getting-started/command-line-options.html).

### Keybindings

You can see the keybindings by pressing the `?` key.

The [default key bindings](https://lusingander.github.io/serie/keybindings/index.html) can be overridden. See [Custom Keybindings](https://lusingander.github.io/serie/keybindings/custom-keybindings.html) for more information.

### Config

Config files are loaded in the following order of priority:

- `$SERIE_CONFIG_FILE`
  - If `$SERIE_CONFIG_FILE` is set but the file does not exist, an error occurs.
- `$XDG_CONFIG_HOME/serie/config.toml`
  - If `$XDG_CONFIG_HOME` is not set, `~/.config/` will be used instead.

If the config file does not exist, the default values will be used for all items.
If the config file exists but some items are not set, the default values will be used for those unset items.

For detailed information about the config file format, see [Config File Format](https://lusingander.github.io/serie/configurations/config-file-format.html).

### User command

The User command feature allows you to execute custom external commands.
You can display the output of commands like `git diff` in a dedicated view, execute commands like branch deletion in the background, or run interactive commands like `vim` by suspending the application.

For details on how to set commands, see [User Command](https://lusingander.github.io/serie/features/user-command.html).

## Compatibility

### Supported terminals

These image protocols are supported:

- [Inline Images Protocol (iTerm2)](https://iterm2.com/documentation-images.html)
- [Terminal graphics protocol (kitty)](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
  - Supports both the existing graphics protocol mode and the [Unicode placeholder](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders) mode.

For more information, see [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html).

### Partially supported environments

- tmux is supported only when using the kitty Unicode placeholder protocol.

### Unsupported environments

- Sixel graphics is not supported.
- Other terminal multiplexers (screen, Zellij, etc.) other than those listed in [Partially supported environments](#partially-supported-environments) are not supported.
- Windows is not officially supported. Please refer to [the related issue](https://github.com/lusingander/serie/issues/147#issuecomment-4192875627).

## Screenshots

<img src="./img/list.png" width=600>
<img src="./img/detail.png" width=600>
<img src="./img/refs.png" width=600>
<img src="./img/searching.png" width=600>
<img src="./img/applied.png" width=600>
<img src="./img/diff_git.png" width=600>
<img src="./img/diff_difft.png" width=600>

The following repositories are used as these examples:

- [ratatui/ratatui](https://github.com/ratatui/ratatui)
- [charmbracelet/vhs](https://github.com/charmbracelet/vhs)
- [lusingander/stu](https://github.com/lusingander/stu)

## Contributing

To get started with contributing, please review [CONTRIBUTING.md](CONTRIBUTING.md).

Contributions that do not follow these guidelines may not be accepted.

## License

MIT
