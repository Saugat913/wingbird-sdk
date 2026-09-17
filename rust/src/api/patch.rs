use std::{
    fs,
    path::{Path, PathBuf},
};

use flutter_rust_bridge::frb;

use crate::patch::{
    api_client::ApiClient,
    operations::{apply_patch, hash_file, verify_file_hash},
    types::{LibCache, LibVersion, PatchConfig},
};

pub struct WingbirdPatchManagerConfig {
    pub server_url: String,

    pub app_id: String,
    pub version: String,
    pub channel: String,
    pub platform: String,
    pub architecture: String,

    pub root_path: String,
    pub native_libapp_path: String,
}

#[frb(opaque)]
pub struct WingbirdPatchManager {
    config: WingbirdPatchManagerConfig,
    client: ApiClient,
}

impl WingbirdPatchManager {
    pub fn new(config: WingbirdPatchManagerConfig) -> Self {
        let server_url = config.server_url.clone();
        Self {
            config,
            client: ApiClient::new(server_url).unwrap(),
        }
    }

    fn root(&self) -> &Path {
        Path::new(&self.config.root_path)
    }

    fn patch_dir(&self) -> PathBuf {
        self.root().join("patch")
    }

    fn lib_dir(&self) -> PathBuf {
        self.root().join("lib")
    }

    fn patch_path(&self) -> PathBuf {
        self.patch_dir().join("libapp.patch")
    }

    fn temp_patch_path(&self) -> PathBuf {
        self.patch_path().with_extension(".tmp")
    }

    fn config_path(&self) -> PathBuf {
        self.patch_dir().join(".config")
    }

    fn libapp_path(&self) -> PathBuf {
        self.lib_dir().join("libapp.so")
    }
    fn temp_libapp_path(&self) -> PathBuf {
        self.libapp_path().with_extension(".tmp")
    }

    fn cache_path(&self) -> PathBuf {
        self.lib_dir().join(".cache")
    }

    fn version_path(&self) -> PathBuf {
        self.lib_dir().join(".version")
    }

    fn native_libapp_path(&self) -> PathBuf {
        PathBuf::from(&self.config.native_libapp_path)
    }

    fn ensure_dirs(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self.patch_dir())?;
        fs::create_dir_all(self.lib_dir())?;
        Ok(())
    }

    fn remove_if_exists(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn cleanup_patch_folder(&self) -> anyhow::Result<()> {
        self.remove_if_exists(self.temp_patch_path())?;
        self.remove_if_exists(self.patch_path())?;
        self.remove_if_exists(self.config_path())?;
        Ok(())
    }

    fn cleanup_lib_folder(&self) -> anyhow::Result<()> {
        self.remove_if_exists(self.cache_path())?;
        self.remove_if_exists(self.temp_libapp_path())?;
        self.remove_if_exists(self.libapp_path())?;
        Ok(())
    }

    fn read_lib_version_from_file(&self) -> anyhow::Result<LibVersion> {
        Ok(LibVersion::from_file(self.version_path())?)
    }

    fn read_cache(&self) -> anyhow::Result<LibCache> {
        Ok(serde_json::from_slice(&fs::read(self.cache_path())?)?)
    }

    fn read_patch_config(&self) -> anyhow::Result<PatchConfig> {
        Ok(serde_json::from_slice(&fs::read(self.config_path())?)?)
    }

    // Recover if the library libapp is broken or not and verify it
    fn recover_from_lib_broken(&self) -> anyhow::Result<()> {
        self.ensure_dirs()?;

        let version_exists = self.version_path().exists();
        let cache_exists = self.cache_path().exists();
        let temp_lib_exists = self.temp_libapp_path().exists();
        let libapp_exists = self.libapp_path().exists();

        if version_exists {
            let version_config = match self.read_lib_version_from_file() {
                Ok(config) => config,
                Err(e) => {
                    log::error!("Failed to read version file: {}", e);
                    self.cleanup_lib_folder()?;
                    return Ok(());
                }
            };

            // That means app is already upadated
            if version_config.release_version != self.config.version {
                return self.cleanup_lib_folder();
            }

            if cache_exists {
                self.remove_if_exists(self.cache_path())?;
                return Ok(());
            }

            return Ok(());
        }

        // No version means commit did not happen.
        if cache_exists {
            let cache = match self.read_cache() {
                Ok(cache) => cache,
                Err(e) => {
                    log::error!("Failed to read cache: {e}");

                    self.remove_if_exists(self.cache_path())?;
                    self.remove_if_exists(self.temp_libapp_path())?;

                    return Ok(());
                }
            };

            if cache.release_version != self.config.version {
                self.remove_if_exists(self.cache_path())?;
                self.remove_if_exists(self.temp_libapp_path())?;

                return Ok(());
            }

            if libapp_exists {
                let hash = hash_file(self.libapp_path())?;
                if hash.to_hex().to_string() == cache.libapp_hash {
                    let version = LibVersion {
                        release_version: cache.release_version,
                        patch_number: cache.patch_number,
                    };
                    version.to_file(self.version_path().to_str().expect("path"))?;
                    self.remove_if_exists(self.cache_path())?;
                    self.remove_if_exists(self.temp_libapp_path())?;
                    return Ok(());
                }
            }
            if temp_lib_exists {
                let hash = hash_file(self.temp_libapp_path())?;

                if hash.to_hex().to_string() == cache.libapp_hash {
                    fs::rename(self.temp_libapp_path(), self.libapp_path())?;

                    let version = LibVersion {
                        release_version: cache.release_version,
                        patch_number: cache.patch_number,
                    };

                    version.to_file(
                        self.version_path()
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid version path"))?,
                    )?;

                    self.remove_if_exists(self.cache_path())?;

                    return Ok(());
                }
                self.remove_if_exists(self.temp_libapp_path())?;
            }
        }

        self.cleanup_lib_folder()
    }

    fn recover_from_patch_broken(&self) -> anyhow::Result<()> {
        let patch_exists = self.patch_path().exists();
        let config_exists = self.config_path().exists();

        if patch_exists && config_exists {
            let config = match self.read_patch_config() {
                Ok(config) => config,
                Err(e) => {
                    log::error!("Failed to read patch config: {}", e);
                    self.cleanup_patch_folder()?;
                    return Ok(());
                }
            };

            if config.release_version != self.config.version {
                self.cleanup_patch_folder()?;
                return Ok(());
            }

            let patch_hash = hash_file(self.patch_path())?;

            if patch_hash.to_hex().to_string() != config.patch_hash {
                self.cleanup_patch_folder()?;
                return Ok(());
            }
            if let Err(e) = self.apply_patch(
                config.libapp_hash.clone(),
                config.patch_number,
                config.release_version,
            ) {
                log::error!("Failed to resume patch apply: {}", e);
                self.cleanup_patch_folder()?;
            }
        }
        Ok(())
    }

    fn apply_patch(
        &self,
        libapp_hash: String,
        patch_number: u32,
        version: String,
    ) -> anyhow::Result<()> {
        let version_config_exists = self.version_path().exists();

        let cache_config = LibCache {
            libapp_hash: libapp_hash.clone(),
            release_version: version.clone(),
            patch_number: patch_number,
        };

        log::info!(
            "[Wingbird Rust] Writing cache config to {}",
            self.cache_path().display()
        );
        fs::write(self.cache_path(), serde_json::to_string(&cache_config)?)?;

        let patched_hash = apply_patch(
            &self.patch_path(),
            &self.native_libapp_path(),
            &self.temp_libapp_path(),
        )?;

        if patched_hash != libapp_hash {
            return Err(anyhow::anyhow!("Patch verification failed"));
        }
        if version_config_exists {
            self.remove_if_exists(self.version_path())?;
        }
        fs::rename(&self.temp_libapp_path(), &self.libapp_path())?;

        let version_config = LibVersion {
            release_version: version,
            patch_number: patch_number,
        };

        log::info!(
            "[Wingbird Rust] Writing version config to {}",
            self.version_path().display()
        );
        version_config.to_file(
            self.version_path()
                .to_str()
                .expect("Failed to convert path to string"),
        )?;

        self.remove_if_exists(self.cache_path())?;
        self.cleanup_patch_folder()?;
        Ok(())
    }

    pub fn get_current_patch_number(&self) -> u32 {
        let version_path = self.version_path();
        if !version_path.exists() {
            return 0;
        }
        let version_config = LibVersion::from_file(
            version_path
                .to_str()
                .expect("Failed to convert path to string"),
        );
        return match version_config {
            Ok(config) => config.patch_number,
            Err(_) => 0,
        };
    }
    
    pub fn run(&self) -> anyhow::Result<()> {
        self.recover_from_lib_broken()?;

        let current_patch_number = self.get_current_patch_number();

        let path_meta = match self.client.check_latest_patch(
            self.config.app_id.clone(),
            self.config.version.clone(),
            self.config.channel.clone(),
            self.config.platform.clone(),
            self.config.architecture.clone(),
            current_patch_number,
        ) {
            Ok(meta) => meta,
            Err(e) => {
                // Mostly cannot connect to the server or server returned an error
                // if we cannot connect to the error, we should not fail the whole process
                // just recover_from_patch_broken
                log::error!("Failed to check for latest patch: {}", e);
                return self.recover_from_patch_broken();
            }
        };

        if let Some(patch_meta) = path_meta {
            self.remove_if_exists(self.temp_patch_path())?;
            self.remove_if_exists(self.config_path())?;
            self.remove_if_exists(self.patch_path())?;

            let patch_config = PatchConfig {
                patch_number: patch_meta.patch_number,
                patch_hash: patch_meta.patch_hash.clone(),
                libapp_hash: patch_meta.libapp_hash.clone(),
                release_version: self.config.version.clone(),
            };

            log::info!(
                "[Wingbird Rust] Writing patch config to {}",
                self.config_path().display()
            );
            fs::write(self.config_path(), serde_json::to_string(&patch_config)?)?;

            self.client
                .download_patch(patch_meta.id.clone(), &self.temp_patch_path())?;

            log::info!("[Wingbird Rust] Verifying patch hash...");
            if let Err(e) = verify_file_hash(&self.temp_patch_path(), &patch_meta.patch_hash) {
                return Err(anyhow::anyhow!("Patch verification failed: {}", e));
            }

            fs::rename(&self.temp_patch_path(), &self.patch_path())?;

            self.apply_patch(
                patch_meta.libapp_hash.clone(),
                patch_meta.patch_number,
                self.config.version.clone(),
            )?;
        } else {
            return self.recover_from_patch_broken();
        }

        Ok(())
    }
}
