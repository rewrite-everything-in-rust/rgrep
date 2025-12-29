# rgrep

A fast, memory-mapped implementation of `grep` written in Rust. `rgrep` provides a subset of the functionality of the classic `grep` utility, focusing on performance by using memory-mapped files and the efficient `regex` crate.

## Features

- **Blazing Fast**: Uses `memmap2` to map files into memory for high-performance searching.
- **Full Regex Support**: Powered by the `regex` crate for efficient pattern matching.
- **Standard Grep Flags**: Supports common flags like `-i` (ignore-case), `-v` (invert-match), `-r` (recursive), and more.
- **Colorized Output**: Automatic color detection or manual control with `--color`.
- **Recursive Search**: Search through directories and subdirectories with ease.
- **Stdin Support**: Pipe input directly into `rgrep` for quick filtering.

## Installation

To build and install `rgrep` from source, ensure you have [Rust and Cargo](https://rustup.rs/) installed:

```bash
cargo install --path .
```

Alternatively, you can build the release binary:

```bash
cargo build --release
```

The resulting binary will be located at `target/release/rgrep`.

## Usage

```bash
rgrep [OPTIONS] PATTERN [FILE...]
```

### Examples

Search for a pattern in a file:
```bash
rgrep "TODO" src/main.rs
```

Search recursively through a directory:
```bash
rgrep -r "fn main" .
```

Case-insensitive search with line numbers:
```bash
rgrep -in "rust" README.md
```

Count matches in multiple files:
```bash
rgrep -c "pub" src/*.rs
```

Pipe from another command:
```bash
ls -l | rgrep "drwx"
```

## Supported Options

| Option | Long Flag | Description |
|--------|-----------|-------------|
| `-i` | `--ignore-case` | Ignore case distinctions |
| `-v` | `--invert-match` | Select non-matching lines |
| `-c` | `--count` | Print only a count of matching lines |
| `-n` | `--line-number` | Print line numbers |
| `-l` | `--files-with-matches` | Print only names of files with matches |
| `-L` | `--files-without-match` | Print only names of files without matches |
| `-h` | `--no-filename` | Suppress file name prefix |
| `-H` | `--with-filename` | Print file name for each match |
| `-o` | `--only-matching` | Show only matching parts of lines |
| `-q` | `--quiet` | Suppress all output |
| `-r` | `--recursive` | Search directories recursively |
| `-F` | `--fixed-strings` | Interpret pattern as fixed strings |
| `-w` | `--word-regexp` | Match whole words only |
| `-x` | `--line-regexp` | Match whole lines only |
| `-b` | `--byte-offset` | Print byte offset of matches |
| `-m` | `--max-count NUM` | Stop after NUM matches |
| | `--color` | Use colors in output |

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.
