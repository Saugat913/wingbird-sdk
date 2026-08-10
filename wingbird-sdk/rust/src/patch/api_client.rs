use std::path::Path;

use crate::patch::types::PatchMetadata;
use anyhow::bail;
use reqwest::blocking::Client;

pub struct ApiClient {
    client: Client,
    server_url: String,
}

impl ApiClient {
    pub fn new(server_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().user_agent("wingbird-sdk/0.1.0").build()?,
            server_url,
        })
    }

    pub fn check_latest_patch(
        &self,
        app_id: String,
        version: String,
        channel: String,
        platform: String,
        architecture: String,
        current_patch_number: u32,
    ) -> anyhow::Result<Option<PatchMetadata>> {
        let url = format!(
            "{}/api/apps/{}/releases/{}/patches/latest?platform={}&channel={}&architecture={}&currentPatchNumber={}",
            self.server_url.trim_end_matches('/'),
            urlencoding::encode(&app_id),
            urlencoding::encode(&version),
            urlencoding::encode(&platform),
            urlencoding::encode(&channel),
            urlencoding::encode(&architecture),
            current_patch_number,
        );

        log::info!("[Wingbird Rust] Checking for patch at URL: {}", url);

        let response = self.client.get(&url).send()?;
        log::info!("[Wingbird Rust] Check patch status: {}", response.status());

        match response.status() {
            reqwest::StatusCode::OK => {
                log::info!("[Wingbird Rust] Patch available");
                let patch_metadata = response.json::<PatchMetadata>()?;
                log::info!("[Wingbird Rust] Patch metadata: {:?}", patch_metadata);
                return Ok(Some(patch_metadata));
            }
            reqwest::StatusCode::NOT_FOUND => {
                log::info!("[Wingbird Rust] No newer patch available");
                return Ok(None);
            }
            status => {
                return Err(anyhow::anyhow!("HTTP {}: {}", status, response.text()?));
            }
        }
    }

    pub fn download_patch(&self, patch_id: String, download_path: &Path) -> anyhow::Result<()> {
        let url = format!(
            "{}/api/patches/{}/download",
            self.server_url.trim_end_matches('/'),
            patch_id
        );

        log::info!("[Wingbird Rust] Requesting patch download URL: {}", url);

        let response = self.client.get(&url).send()?;
        log::info!(
            "[Wingbird Rust] Download endpoint status: {}",
            response.status()
        );

        if !response.status().is_success() {
            bail!(
                "Download failed with HTTP {}: {}",
                response.status(),
                response.text()?
            )
        }

        let bytes = response.bytes()?;
        std::fs::write(download_path, &bytes)?;
        log::info!(
            "[Wingbird Rust] Patch downloaded ({} bytes) to {}",
            bytes.len(),
            download_path.display()
        );

        Ok(())
    }
}
