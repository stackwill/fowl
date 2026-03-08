mod extract;
mod download;

use clap::Parser;
use std::path::PathBuf;

// Embed aria2c at compile time
const ARIA2C_BYTES: &[u8] = include_bytes!(env!("ARIA2C_PATH"));

#[derive(Parser)]
#[command(name = "fowl", about = "Terminal download accelerator")]
struct Args {
    /// URL to download
    url: String,

    /// Output file path
    #[arg(short = 'o')]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let (_tempdir, aria2c_path) = extract::extract_aria2c(ARIA2C_BYTES)?;
    download::run(&aria2c_path, &args.url, args.output.as_deref())?;
    Ok(())
}
