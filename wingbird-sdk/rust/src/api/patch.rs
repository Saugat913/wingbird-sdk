use std::{
    fs,
    io::Cursor,
};

use anyhow::{anyhow, bail, Result};
use flutter_rust_bridge::frb;
use qbsdiff::Bspatch;
use reqwest::{
    blocking::Client,
    StatusCode,
};

/// Returns true if patch is downloaded, false if no patch is available
#[frb]
pub fn download_patch(
    server_url: String,
    app_id: String,
    channel: String,
    current_version: String,
    architecture: String,
    platform: String,
    download_path: String,
) -> Result<bool> {
    let encoded_version = urlencoding::encode(&current_version);
    let encoded_platform = urlencoding::encode(&platform);
    let encoded_channel = urlencoding::encode(&channel);
    let encoded_architecture = urlencoding::encode(&architecture);

    let url = format!(
        "{}/api/apps/{}/releases/{}/patches/latest/download\
?platform={}&channel={}&architecture={}",
        server_url.trim_end_matches('/'),
        app_id,
        encoded_version,
        encoded_platform,
        encoded_channel,
        encoded_architecture,
    );

    log::info!("[Wingbird Rust] Requesting patch from URL: {}", url);

    let client = Client::builder()
        .user_agent("wingbird-sdk/0.1.0")
        .build()?;

    let response = client.get(url).send()?;
    log::info!("[Wingbird Rust] Patch server response status: {}", response.status());

    match response.status() {
        StatusCode::OK => {
            let bytes = response.bytes()?;
            log::info!("[Wingbird Rust] Patch downloaded successfully, size: {} bytes. Saving to: {}", bytes.len(), download_path);
            fs::write(download_path, bytes)?;
            Ok(true)
        }

        StatusCode::NOT_FOUND => {
            log::info!("[Wingbird Rust] No patch available (404 Not Found).");
            Ok(false)
        }

        StatusCode::BAD_REQUEST => {
            let body = response.text()?;
            bail!("Bad request: {}", body);
        }

        StatusCode::UNAUTHORIZED => {
            let body = response.text()?;
            bail!("Unauthorized: {}", body);
        }

        StatusCode::FORBIDDEN => {
            let body = response.text()?;
            bail!("Forbidden: {}", body);
        }

        StatusCode::INTERNAL_SERVER_ERROR => {
            let body = response.text()?;
            bail!("Internal server error: {}", body);
        }

        status => {
            let body = response.text()?;
            Err(anyhow!("HTTP {}: {}", status, body))
        }
    }
}

#[frb]
pub fn apply_patch(
    patch_file_path: String,
    initial_version_file_path: String,
    final_version_file_path: String,
) -> Result<()> {
    log::info!("[Wingbird Rust] Applying patch. Patch: {}, Initial: {}, Final: {}", patch_file_path, initial_version_file_path, final_version_file_path);

    let source = fs::read(initial_version_file_path)?;
    let patch = fs::read(patch_file_path)?;

    let patcher = Bspatch::new(&patch)?;

    let mut output = Vec::with_capacity(
        patcher.hint_target_size() as usize,
    );

    patcher.apply(&source, Cursor::new(&mut output))?;

    fs::write(&final_version_file_path, output)?;
    log::info!("[Wingbird Rust] Patch applied and saved successfully to: {}", final_version_file_path);

    Ok(())
}
