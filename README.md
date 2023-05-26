# DragonRuby Environment Manager

![Rust action status](https://github.com/petros/drem/actions/workflows/rust.yml/badge.svg)

This is a command line utility that helps with the following tasks:

1. Start a new game that can be published as an open source project
2. Clone an existing open source game within a DRGTK folder

## Assumptions

- The developer (you) has already downloaded a version of DRGTK somewhere on their system.
- `drem` only works on macOS for now

## Usage

```shell
drem new <game-name> --drgtk <path-to-drgtk>
```
