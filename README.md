# `drem` (DragonRuby Environment Manager)

![Rust action status](https://github.com/petros/drem/actions/workflows/rust.yml/badge.svg) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://makeapullrequest.com)

A command line utility written in Rust to help manage DragonRuby projects. This is related to the [DragonRuby Game Toolkit](https://dragonruby.org/toolkit/game) (DRGTK).

## Usage

### General help

```
$ drem --help
Usage: drem [COMMAND]

Commands:
  new   Create a new game
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Create a new game

It will create a local copy of DRGTK and initialize a new game under it in a way that will make it easy to push to GitHub or elsewhere.

```
$ drem new --help
Create a new game

Usage: drem new --name <name> --drgtk <drgtk>

Options:
  -n, --name <name>    Name of the new game
  -d, --drgtk <drgtk>  Path to DRGTK zip to use
  -h, --help           Print help
```

#### Example

```
$ drem new -n eggs -d ~/Downloads/dragonruby-game-toolkit-macos.zip
```

This will do the following:
- Check if the referenced DRGTK zip file is a legitimate DRGTK.
- Unzip the DRGTK zip file to a folder called `dragonruby-eggs-drgtk` in the current directory.
- Add a `.gitkeep` file under `dragonruby-eggs-drgtk/mygame/data`, `dragonruby-eggs-drgtk/mygame/fonts`, and `dragonruby-eggs-drgtk/mygame/sounds`. This ensures that these folders are tracked by Git as they are empty by default.
- Add a `.gitignore` file under `dragonruby-eggs-drgtk/mygame` that ignores `.DS_Store` files.
- Initializes a git repository under `dragonruby-eggs-drgtk/mygame`.

After that, the developer can start working on their game and push `dragonruby-eggs-drgtk/mygame` to GitHub or elsewhere.

## Assumptions

- The developer has already downloaded a version of DRGTK somewhere on their system.
- `drem` only works on macOS for now but should be easy to port to Windows and Linux.

## Contributing

Contributions are welcome. Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for more information.
