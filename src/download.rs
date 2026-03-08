use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ── RPC response shapes ─────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Deserialize, Debug)]
struct RpcError {
    message: String,
}

// ── Drop guard: kills aria2c subprocess on any exit path ────────────────────

struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }
    fn get_mut(&mut self) -> &mut Child {
        self.0.as_mut().unwrap()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("Failed to bind random port")?;
    let port = listener.local_addr()?.port();
    // Keep listener alive until after aria2c is spawned (see call site)
    // so the port cannot be stolen between our bind and aria2c's bind.
    // We return the port and drop the listener just before spawning.
    drop(listener);
    Ok(port)
}

fn random_token() -> String {
    let mut rng = rand::thread_rng();
    (0..24)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

/// Makes a JSON-RPC call to aria2. The `secret` is automatically prepended
/// as the first element of `params` (aria2 token auth convention).
/// `params` must be a JSON array of the non-token arguments.
fn rpc_call(url: &str, secret: &str, method: &str, params: Value) -> Result<Value> {
    let full_params = match params {
        Value::Array(mut arr) => {
            arr.insert(0, json!(format!("token:{secret}")));
            Value::Array(arr)
        }
        other => json!([format!("token:{secret}"), other]),
    };
    let body = json!({
        "jsonrpc": "2.0",
        "id": "fowl",
        "method": method,
        "params": full_params,
    });
    let resp: RpcResponse = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .context("RPC call failed")?
        .into_json()
        .context("Failed to parse RPC response")?;
    if let Some(err) = resp.error {
        return Err(anyhow!("aria2 RPC error: {}", err.message));
    }
    resp.result.ok_or_else(|| anyhow!("RPC response missing result"))
}

fn wait_for_rpc(rpc_url: &str, secret: &str) -> Result<()> {
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(100));
        if rpc_call(rpc_url, secret, "aria2.getVersion", json!([])).is_ok() {
            return Ok(());
        }
    }
    Err(anyhow!("aria2c RPC did not become ready in time"))
}

fn format_bytes(n: u64) -> String {
    if n >= 1_073_741_824 {
        format!("{:.1} GB", n as f64 / 1_073_741_824.0)
    } else if n >= 1_048_576 {
        format!("{:.1} MB", n as f64 / 1_048_576.0)
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} B")
    }
}

// ── Public entry point ───────────────────────────────────────────────────────

pub fn run(aria2c: &Path, url: &str, output: Option<&Path>) -> Result<()> {
    let port = free_port()?;
    let secret = random_token();
    let rpc_url = format!("http://127.0.0.1:{port}/jsonrpc");

    // Determine output dir and filename.
    // Path::parent() returns Some("") for a bare filename like "foo.zip", not None,
    // so we must filter out the empty case explicitly.
    let (out_dir, out_file): (PathBuf, Option<String>) = if let Some(p) = output {
        let dir = p
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .map(|d| d.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned());
        (dir, name)
    } else {
        (PathBuf::from("."), None)
    };

    // Install Ctrl-C handler: sets a flag checked in the poll loop.
    let interrupted = Arc::new(AtomicBool::new(false));
    let flag = interrupted.clone();
    let _ = ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst));

    let mut cmd = Command::new(aria2c);
    cmd.args([
        "--enable-rpc",
        "--rpc-listen-all=false",
        &format!("--rpc-listen-port={port}"),
        &format!("--rpc-secret={secret}"),
        "--quiet",
        "--max-connection-per-server=16",
        "--split=16",
        "--min-split-size=1M",
        "--disk-cache=64M",
        "--file-allocation=none",
        "--continue=true",
        "--max-tries=10",
        "--retry-wait=1",
    ]);
    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    let child = cmd.spawn().context("Failed to spawn aria2c")?;
    // ChildGuard kills and waits on drop, covering panics and early returns.
    let mut guard = ChildGuard::new(child);

    let result = run_download(
        &rpc_url,
        &secret,
        url,
        &out_dir,
        out_file.as_deref(),
        guard.get_mut(),
        &interrupted,
    );

    // Graceful shutdown: ask aria2c to stop, then the guard kills if it hasn't exited.
    let _ = rpc_call(&rpc_url, &secret, "aria2.shutdown", json!([]));
    thread::sleep(Duration::from_millis(300));
    // guard drops here → kill + wait

    result
}

fn run_download(
    rpc_url: &str,
    secret: &str,
    url: &str,
    out_dir: &Path,
    out_file: Option<&str>,
    child: &mut Child,
    interrupted: &AtomicBool,
) -> Result<()> {
    wait_for_rpc(rpc_url, secret)?;

    // Build per-download options (dir/out must be passed here when using RPC,
    // not as CLI flags, because CLI flags apply only to CLI-added downloads).
    let mut dl_opts = serde_json::Map::new();
    dl_opts.insert("dir".into(), json!(out_dir.to_string_lossy().as_ref()));
    if let Some(name) = out_file {
        dl_opts.insert("out".into(), json!(name));
    }

    let gid = rpc_call(rpc_url, secret, "aria2.addUri", json!([[url], dl_opts]))?
        .as_str()
        .ok_or_else(|| anyhow!("addUri did not return a GID string"))?
        .to_owned();

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}  [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}  {binary_bytes_per_sec}  ETA {eta}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );

    loop {
        if interrupted.load(Ordering::SeqCst) {
            pb.abandon();
            return Err(anyhow!("Interrupted"));
        }

        if let Ok(Some(status)) = child.try_wait() {
            return Err(anyhow!("aria2c exited unexpectedly with status {status}"));
        }

        let status_val = rpc_call(
            rpc_url,
            secret,
            "aria2.tellStatus",
            json!([gid, ["status", "totalLength", "completedLength", "errorMessage", "files"]]),
        )?;

        let status = status_val["status"].as_str().unwrap_or("").to_owned();
        let total: u64 = status_val["totalLength"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let completed: u64 = status_val["completedLength"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let filename: String = status_val["files"]
            .as_array()
            .and_then(|files| files.first())
            .and_then(|f| f["path"].as_str())
            .and_then(|p| Path::new(p).file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "downloading".to_string());

        if total > 0 {
            pb.set_length(total);
        }
        pb.set_position(completed);
        pb.set_message(filename.clone());

        match status.as_str() {
            "complete" => {
                pb.finish_with_message(format!("{filename} — done"));
                println!("Downloaded: {}", format_bytes(completed));
                return Ok(());
            }
            "error" => {
                pb.abandon();
                let msg = status_val["errorMessage"]
                    .as_str()
                    .unwrap_or("unknown error");
                return Err(anyhow!("Download failed: {msg}"));
            }
            "paused" | "removed" => {
                pb.abandon();
                return Err(anyhow!("Download stopped unexpectedly (status: {status})"));
            }
            _ => {}
        }

        thread::sleep(Duration::from_millis(200));
    }
}
