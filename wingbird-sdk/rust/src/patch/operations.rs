use anyhow::Result;
use qbsdiff::Bspatch;
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

/// Returns true if the file at `file_path` hashes to `expected_hash` (blake3 hex).
pub fn verify_file_hash<P: AsRef<Path>>(file_path: P, expected_hash: &str) -> Result<bool> {
    let hash = hash_file(file_path)?;
    Ok(hash.to_hex().to_string() == expected_hash)
}

pub fn hash_file<P: AsRef<Path>>(file_path: P) -> Result<blake3::Hash> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize())
}

/// Applies the patch atomically: writes the patched lib to a temp file (one
/// directory up alongside the target), hashes it, then renames it over the
/// target. A crash mid-apply leaves the old libapp.so untouched.
///
/// Returns the blake3 hex hash of the produced libapp.
pub fn apply_patch<P: AsRef<Path>>(
    patch_file_path: P,
    current_libapp_path: P,
    new_libapp_path: P,
) -> Result<String> {
    let patch_file_path = patch_file_path.as_ref();
    let current_libapp_path = current_libapp_path.as_ref();
    let new_libapp_path = new_libapp_path.as_ref();

    log::info!(
        "[Wingbird Rust] Applying patch. Patch: {}, Current: {}, New: {}",
        patch_file_path.display(),
        current_libapp_path.display(),
        new_libapp_path.display()
    );

    let source = fs::read(current_libapp_path)?;
    let patch = fs::read(patch_file_path)?;

    let tmp_path = temp_sibling(new_libapp_path);
    let patched_file = File::create(&tmp_path)?;

    let patcher = Bspatch::new(&patch)?;
    patcher.apply(&source, patched_file)?;

    let hash = hash_file(&tmp_path)?;
    let hex = hash.to_hex().to_string();

    fs::rename(&tmp_path, new_libapp_path)?;

    log::info!(
        "[Wingbird Rust] Patch applied ({} bytes) and atomically replaced {}. libapp hash: {}",
        fs::metadata(new_libapp_path).map(|m| m.len()).unwrap_or(0),
        new_libapp_path.display(),
        hex
    );

    Ok(hex)
}

fn temp_sibling(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target.with_file_name(format!(".{}.new", file_name))
}