use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let aria2c_out = out_dir.join("aria2c");

    // Skip download if already present (incremental builds)
    if !aria2c_out.exists() {
        // Static musl build from abcfy2/aria2-static-build (no official Linux binary in aria2 releases)
        let url = "https://github.com/abcfy2/aria2-static-build/releases/download/1.37.0/aria2-x86_64-linux-musl_static.zip";
        eprintln!("build.rs: Downloading aria2c from {url}");

        let resp = ureq::get(url).call().expect("Failed to download aria2c zip");
        let mut body = Vec::new();
        resp.into_reader().read_to_end(&mut body).expect("Failed to read response body");

        eprintln!("build.rs: Downloaded {} bytes, extracting aria2c binary...", body.len());

        // Extract aria2c from zip
        let cursor = std::io::Cursor::new(&body);
        let mut zip = zip::ZipArchive::new(cursor).expect("Failed to open zip archive");
        let mut found = false;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).expect("Failed to read zip entry");
            let name = file.name().to_owned();
            if std::path::Path::new(&name).file_name().map(|n| n == "aria2c").unwrap_or(false) {
                let mut data = Vec::new();
                file.read_to_end(&mut data).expect("Failed to read aria2c from zip");
                fs::write(&aria2c_out, &data).expect("Failed to write aria2c");
                found = true;
                eprintln!("build.rs: Extracted aria2c ({} bytes) to {}", data.len(), aria2c_out.display());
                break;
            }
        }
        if !found {
            panic!("aria2c binary not found in zip archive");
        }
    } else {
        eprintln!("build.rs: Using cached aria2c at {}", aria2c_out.display());
    }

    println!("cargo:rustc-env=ARIA2C_PATH={}", aria2c_out.display());
    // Re-run only if build.rs itself changes
    println!("cargo:rerun-if-changed=build.rs");
}
