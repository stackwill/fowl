# fowl

Terminal download accelerator. Uses 16 parallel connections to saturate gigabit links.

```
fowl https://example.com/large-file.tar.gz
fowl -o /tmp/out.iso https://example.com/image.iso
```

```
large-file.tar.gz  [=========>    ] 234 MB / 512 MB  45.2 MB/s  ETA 6s
```

## How it works

`fowl` embeds a static [aria2c](https://aria2.github.io/) binary inside the compiled executable. At runtime it extracts the binary to a temporary directory, starts it with RPC enabled, and drives it via JSON-RPC while displaying a progress bar. The final binary is self-contained — no system dependencies required.

- **16 connections / 16 splits** per download
- **Resumes** interrupted downloads automatically (`--continue=true`)
- **64 MB disk cache** to reduce write amplification on SSDs
- **Ctrl-C** cleanly shuts down aria2c

## Requirements

- Linux x86\_64
- Internet access at build time (to download the aria2c static binary)
- Rust / Cargo (install script handles this automatically)

## Install

```bash
git clone https://github.com/your-username/fowl
cd fowl
bash install.sh
```

`install.sh` will:
1. Install `rustup` if `cargo` is not found
2. Build in release mode (downloads aria2c on first build, ~30s)
3. Install to `/usr/local/bin/fowl` (or `~/.local/bin/fowl` if sudo is unavailable)

## Uninstall

```bash
bash uninstall.sh
```

## Manual build

```bash
cargo build --release
./target/release/fowl --help
```

## License

MIT
