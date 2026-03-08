# fowl

Terminal download accelerator. Uses 16 parallel connections to saturate gigabit links.

```
fowl https://example.com/large-file.tar.gz
fowl -o /tmp/out.iso https://example.com/image.iso
```

```
large-file.tar.gz  [=========>    ] 234 MB / 512 MB  45.2 MB/s  ETA 6s
```

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/stackwill/fowl/main/install.sh | bash
```

Requires Linux x86\_64. Installs to `/usr/local/bin/fowl` (or `~/.local/bin/fowl` if sudo is unavailable).

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/stackwill/fowl/main/uninstall.sh | bash
```

## Usage

```
fowl <URL> [-o <output-path>]
```

| Example | Description |
|---------|-------------|
| `fowl https://example.com/file.zip` | Download to current directory |
| `fowl -o /tmp/file.zip https://example.com/file.zip` | Download to specific path |

Interrupted downloads resume automatically on re-run.

## How it works

`fowl` embeds a static [aria2c](https://aria2.github.io/) binary inside the executable — no system dependencies required. At runtime it extracts it to a temp directory, starts it with RPC enabled, and drives it via JSON-RPC while displaying a live progress bar.

- **16 connections / 16 splits** per server
- **64 MB disk cache** to reduce SSD write amplification
- **Resumes** interrupted downloads automatically
- **Ctrl-C** cleanly shuts down aria2c

## Build from source

Requires Rust and a C toolchain (`gcc` / `cc`).

```bash
git clone https://github.com/stackwill/fowl
cd fowl
cargo build --release
# binary at target/release/fowl
```

First build downloads a static aria2c binary (~4 MB) and embeds it. Subsequent builds use the cached copy.

## Releasing a new binary

Build on a Linux x86\_64 machine, then upload `target/release/fowl` as a release asset named `fowl`:

```bash
cargo build --release
gh release create v0.1.0 target/release/fowl --title "v0.1.0"
```

## License

MIT
